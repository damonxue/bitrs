use crate::events::EventHandler;
use crate::orderbook::{Order, OrderBook, OrderType, Side};
use crate::risk::RiskEngine;
use crate::storage::OptimizedStorage;
use crate::ErrorCode;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

// 自成交行为枚举
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq)]
pub enum SelfTradeBehavior {
    DecrementTake,    // 减少吃单数量
    CancelProvide,    // 取消挂单
    AbortTransaction, // 中止交易
}

// 市场状态
#[account]
pub struct Market {
    pub base_mint: Pubkey,              // 基础代币铸币权
    pub quote_mint: Pubkey,             // 报价代币铸币权
    pub lot_size: u64,                  // 最小交易量单位
    pub tick_size: u64,                 // 最小价格变动单位
    pub base_decimals: u8,              // 基础代币小数位数
    pub quote_decimals: u8,             // 报价代币小数位数
    pub maker_fee: i64,                 // 做市商费率，负值表示返佣
    pub taker_fee: i64,                 // 吃单方费率
    pub base_vault: Pubkey,             // 基础代币保管库
    pub quote_vault: Pubkey,            // 报价代币保管库
    pub lp_token_mint: Option<Pubkey>,  // 流动性代币铸币权
    pub reward_mint: Option<Pubkey>,    // 奖励代币铸币权
    pub name: String,                   // 交易对名称
    pub active: bool,                   // 市场是否活跃
    pub last_traded_price: Option<u64>, // 最后成交价
    pub total_volume: u64,              // 总成交量
    pub total_fees_collected: u64,      // 总手续费收入
    pub min_base_order_size: u64,       // 最小基础代币订单大小
    pub min_quote_order_size: u64,      // 最小报价代币订单大小
    pub lp_reward_rate: u64,            // 流动性奖励率（每块）
    pub cross_chain_enabled: bool,      // 是否启用跨链交易
    pub stress_test_mode: bool,         // 压力测试模式
    pub market_authority_bump: u8,      // 市场权限PDA的bump
}

impl Market {
    pub const LEN: usize = 32
        + 32
        + 8
        + 8
        + 1
        + 1
        + 8
        + 8
        + 32
        + 32
        + (1 + 32)
        + (1 + 32)
        + 32
        + 1
        + 9
        + 8
        + 8
        + 8
        + 8
        + 8
        + 1
        + 1
        + 1;
}

// 用户的开放订单账户
#[account]
pub struct OpenOrders {
    pub owner: Pubkey,              // 所有者公钥
    pub market: Pubkey,             // 所属市场
    pub locked_base_tokens: u64,    // 锁定的基础代币数量
    pub locked_quote_tokens: u64,   // 锁定的报价代币数量
    pub filled_base_quantity: u64,  // 已成交的基础代币数量
    pub filled_quote_quantity: u64, // 已成交的报价代币数量
    pub fees_paid: u64,             // 已支付手续费
    pub lp_tokens_earned: u64,      // 已获得的LP代币数量
    pub reward_tokens_earned: u64,  // 已获得的奖励代币数量
    pub last_update_slot: u64,      // 上次更新的slot
    pub strategies_count: u8,       // 用户设置的策略数量
    pub bump: u8,                   // PDA bump值
    // 压缩存储的订单信息
    pub orders_bitmap: [u64; 4], // 使用位图快速查找订单
    pub order_count: u16,        // 当前订单数量
}

impl OpenOrders {
    pub const LEN: usize = 32 + 32 + 8 + 8 + 8 + 8 + 8 + 8 + 8 + 8 + 1 + 1 + (8 * 4) + 2;
}

// Anchor账户验证结构定义
#[derive(Accounts)]
pub struct InitializeMarket<'info> {
    #[account(init, payer = authority, space = 8 + Market::LEN)]
    pub market: Account<'info, Market>,
    pub base_mint: Account<'info, token::Mint>,
    pub quote_mint: Account<'info, token::Mint>,
    #[account(
        init,
        payer = authority,
        token::mint = base_mint,
        token::authority = market_authority,
    )]
    pub base_vault: Account<'info, TokenAccount>,
    #[account(
        init,
        payer = authority,
        token::mint = quote_mint,
        token::authority = market_authority,
    )]
    pub quote_vault: Account<'info, TokenAccount>,
    #[account(
        seeds = [b"market_authority", market.key().as_ref()],
        bump
    )]
    /// CHECK: 这是一个PDA，不需要验证
    pub market_authority: UncheckedAccount<'info>,
    #[account(
        init,
        payer = authority,
        space = 8 + OrderBook::LEN
    )]
    pub order_book: AccountLoader<'info, OrderBook>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct PlaceOrder<'info> {
    #[account(mut, has_one = base_vault, has_one = quote_vault)]
    pub market: Account<'info, Market>,
    #[account(mut)]
    pub order_book: AccountLoader<'info, OrderBook>,
    #[account(
        init_if_needed,
        payer = authority,
        space = 8 + OpenOrders::LEN,
        seeds = [b"open_orders", authority.key().as_ref(), market.key().as_ref()],
        bump
    )]
    pub open_orders: Account<'info, OpenOrders>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(signer)]
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct CancelOrder<'info> {
    #[account(mut)]
    pub market: Account<'info, Market>,
    #[account(mut)]
    pub order_book: AccountLoader<'info, OrderBook>,
    #[account(
        mut,
        seeds = [b"open_orders", authority.key().as_ref(), market.key().as_ref()],
        bump
    )]
    pub open_orders: Account<'info, OpenOrders>,
    #[account(signer)]
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct SettleFunds<'info> {
    #[account(mut)]
    pub market: Account<'info, Market>,
    #[account(
        mut,
        seeds = [b"open_orders", authority.key().as_ref(), market.key().as_ref()],
        bump = open_orders.bump
    )]
    pub open_orders: Account<'info, OpenOrders>,
    #[account(
        mut,
        address = market.base_vault
    )]
    pub market_base_vault: Account<'info, TokenAccount>,
    #[account(
        mut,
        address = market.quote_vault
    )]
    pub market_quote_vault: Account<'info, TokenAccount>,
    #[account(
        seeds = [b"market_authority", market.key().as_ref()],
        bump = market.market_authority_bump
    )]
    /// CHECK: 这是一个PDA，不需要验证
    pub market_authority: UncheckedAccount<'info>,
    #[account(mut)]
    pub user_base_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_quote_account: Account<'info, TokenAccount>,
    #[account(signer)]
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub clock: Sysvar<'info, Clock>,
}

// 核心功能实现
pub fn initialize_market(
    ctx: Context<InitializeMarket>,
    market_name: String,
    base_decimals: u8,
    quote_decimals: u8,
    lot_size: u64,
    tick_size: u64,
    maker_fee: i64,
    taker_fee: i64,
) -> Result<()> {
    let market = &mut ctx.accounts.market;
    market.base_mint = ctx.accounts.base_mint.key();
    market.quote_mint = ctx.accounts.quote_mint.key();
    market.lot_size = lot_size;
    market.tick_size = tick_size;
    market.base_decimals = base_decimals;
    market.quote_decimals = quote_decimals;
    market.name = market_name;
    market.maker_fee = maker_fee;
    market.taker_fee = taker_fee;
    market.base_vault = ctx.accounts.base_vault.key();
    market.quote_vault = ctx.accounts.quote_vault.key();
    market.lp_token_mint = None; // 初始没有LP代币
    market.reward_mint = None; // 初始没有奖励代币
    market.active = true;
    market.total_fees_collected = 0;
    market.min_base_order_size = lot_size; // 默认为lot_size
    market.min_quote_order_size = tick_size; // 默认为tick_size
    market.lp_reward_rate = 0; // 默认无奖励
    market.cross_chain_enabled = false; // 默认禁用跨链
    market.stress_test_mode = false; // 默认非压测模式
    market.market_authority_bump = *ctx.bumps.get("market_authority").unwrap();

    // 初始化订单簿
    let order_book = &mut ctx.accounts.order_book.load_init()?;
    order_book.initialize();

    // 记录市场创建事件
    EventHandler::emit_market_created(
        market.key(),
        ctx.accounts.base_mint.key(),
        ctx.accounts.quote_mint.key(),
        lot_size,
        tick_size,
    );

    Ok(())
}

pub fn place_order(
    ctx: Context<PlaceOrder>,
    side: Side,
    limit_price: u64,
    max_quantity: u64,
    order_type: OrderType,
    self_trade_behavior: SelfTradeBehavior,
    client_order_id: u64,
) -> Result<()> {
    // 检查市场是否活跃
    require!(ctx.accounts.market.active, ErrorCode::MarketNotActive);

    // 检查价格和数量是否有效
    require!(max_quantity > 0, ErrorCode::InvalidOrderQuantity);
    require!(
        limit_price % ctx.accounts.market.tick_size == 0,
        ErrorCode::InvalidOrderPrice
    );

    // 验证最小订单大小
    match side {
        Side::Bid => {
            require!(
                max_quantity * ctx.accounts.market.lot_size
                    >= ctx.accounts.market.min_base_order_size,
                ErrorCode::InvalidOrderQuantity
            );
        }
        Side::Ask => {
            require!(
                max_quantity * limit_price >= ctx.accounts.market.min_quote_order_size,
                ErrorCode::InvalidOrderQuantity
            );
        }
    }

    // 构建订单
    let order_id = generate_order_id(
        &ctx.accounts.authority.key(),
        client_order_id,
        Clock::get()?.slot,
    );

    let order = Order::new(
        order_id,
        ctx.accounts.authority.key(),
        side,
        limit_price,
        max_quantity,
        order_type,
    );

    // 风险检查 - 验证用户资金是否足够
    RiskEngine::check_funds(&ctx.accounts, &order)?;

    // 处理订单 - 先尝试匹配，然后根据订单类型决定是否添加到订单簿
    let order_book = &mut ctx.accounts.order_book.load_mut()?;
    let (trades, remaining_order) = order_book.process_order(order, self_trade_behavior)?;

    // 处理交易结果
    let market = &ctx.accounts.market;
    let open_orders = &mut ctx.accounts.open_orders;

    for trade in &trades {
        // 记录交易事件
        EventHandler::emit_trade(
            market.key(),
            side,
            trade.maker_order_id,
            order_id,
            trade.price,
            trade.quantity,
            trade.maker,
            ctx.accounts.authority.key(),
        );

        // 更新用户的交易统计
        open_orders.filled_base_quantity += trade.base_quantity;
        open_orders.filled_quote_quantity += trade.quote_quantity;

        // 计算手续费
        let taker_fee = if market.taker_fee > 0 {
            // 正值表示收取费用
            (trade.quote_quantity as u128 * market.taker_fee as u128 / 10000) as u64
        } else {
            0
        };

        open_orders.fees_paid += taker_fee;
    }

    // 处理剩余订单
    if let Some(remaining) = remaining_order {
        if remaining.side == Side::Bid {
            // 锁定买单所需的报价代币
            let required_funds = RiskEngine::calculate_required_quote_for_bid(
                remaining.price,
                remaining.quantity,
                market.lot_size,
                market.tick_size,
                market.taker_fee,
            );
            open_orders.locked_quote_tokens += required_funds;
        } else {
            // 锁定卖单所需的基础代币
            let required_base = remaining.quantity * market.lot_size;
            open_orders.locked_base_tokens += required_base;
        }

        // 如果需要将订单添加到用户的开放订单列表
        if order_type == OrderType::Limit || order_type == OrderType::PostOnly {
            add_to_open_orders(open_orders, &remaining)?;

            // 发出挂单事件
            EventHandler::emit_order_placed(
                market.key(),
                remaining.order_id,
                remaining.owner,
                remaining.side,
                remaining.price,
                remaining.quantity,
            );
        }
    }

    // 更新市场统计信息
    if !trades.is_empty() {
        let last_trade = trades.last().unwrap();
        let market = &mut ctx.accounts.market;
        market.last_traded_price = Some(last_trade.price);
        market.total_volume += trades.iter().map(|t| t.base_quantity).sum::<u64>();
    }

    Ok(())
}

pub fn cancel_order(ctx: Context<CancelOrder>, order_id: u128, side: Side) -> Result<()> {
    // 检查市场是否活跃
    require!(ctx.accounts.market.active, ErrorCode::MarketNotActive);

    // 获取开放订单账户
    let open_orders = &mut ctx.accounts.open_orders;

    // 验证用户是否为订单所有者
    require!(
        open_orders.owner == ctx.accounts.authority.key(),
        ErrorCode::UnauthorizedOperation
    );

    // 验证订单是否存在
    require!(
        check_order_exists(open_orders, order_id),
        ErrorCode::OrderNotFound
    );

    // 从订单簿中取消订单
    let order_book = &mut ctx.accounts.order_book.load_mut()?;
    let removed_order = order_book.cancel_order(order_id, side)?;

    // 更新OpenOrders账户并解锁资金
    if side == Side::Bid {
        // 解锁quote tokens
        let locked_amount = RiskEngine::calculate_required_quote_for_bid(
            removed_order.price,
            removed_order.quantity,
            ctx.accounts.market.lot_size,
            ctx.accounts.market.tick_size,
            ctx.accounts.market.taker_fee,
        );
        open_orders.locked_quote_tokens = open_orders
            .locked_quote_tokens
            .saturating_sub(locked_amount);
    } else {
        // 解锁base tokens
        let locked_amount = removed_order.quantity * ctx.accounts.market.lot_size;
        open_orders.locked_base_tokens =
            open_orders.locked_base_tokens.saturating_sub(locked_amount);
    }

    // 从用户的开放订单列表中移除该订单
    remove_from_open_orders(open_orders, order_id)?;

    // 发出取消事件
    EventHandler::emit_order_canceled(
        ctx.accounts.market.key(),
        order_id,
        ctx.accounts.authority.key(),
        side,
        removed_order.price,
        removed_order.quantity,
    );

    Ok(())
}

pub fn settle_funds(ctx: Context<SettleFunds>) -> Result<()> {
    // 检查市场是否活跃
    require!(ctx.accounts.market.active, ErrorCode::MarketNotActive);

    let market = &ctx.accounts.market;
    let open_orders = &mut ctx.accounts.open_orders;

    // 验证用户身份
    require!(
        open_orders.owner == ctx.accounts.authority.key(),
        ErrorCode::UnauthorizedOperation
    );

    // 计算可提取的金额
    let base_to_settle = open_orders
        .filled_base_quantity
        .saturating_sub(open_orders.locked_base_tokens);
    let quote_to_settle = open_orders
        .filled_quote_quantity
        .saturating_sub(open_orders.locked_quote_tokens);

    // 确保有资金可以结算
    require!(
        base_to_settle > 0 || quote_to_settle > 0,
        ErrorCode::NoFundsToSettle
    );

    // 转移base tokens(如果有)
    if base_to_settle > 0 {
        let transfer_base_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.market_base_vault.to_account_info(),
                to: ctx.accounts.user_base_account.to_account_info(),
                authority: ctx.accounts.market_authority.to_account_info(),
            },
        );

        token::transfer(
            transfer_base_ctx.with_signer(&[&[
                b"market_authority",
                market.key().as_ref(),
                &[market.market_authority_bump],
            ]]),
            base_to_settle,
        )?;

        // 更新已结算金额
        open_orders.filled_base_quantity = open_orders
            .filled_base_quantity
            .saturating_sub(base_to_settle);
    }

    // 转移quote tokens(如果有)
    if quote_to_settle > 0 {
        let transfer_quote_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.market_quote_vault.to_account_info(),
                to: ctx.accounts.user_quote_account.to_account_info(),
                authority: ctx.accounts.market_authority.to_account_info(),
            },
        );

        token::transfer(
            transfer_quote_ctx.with_signer(&[&[
                b"market_authority",
                market.key().as_ref(),
                &[market.market_authority_bump],
            ]]),
            quote_to_settle,
        )?;

        // 更新已结算金额
        open_orders.filled_quote_quantity = open_orders
            .filled_quote_quantity
            .saturating_sub(quote_to_settle);
    }

    // 发出结算事件
    EventHandler::emit_funds_settled(
        market.key(),
        ctx.accounts.authority.key(),
        base_to_settle,
        quote_to_settle,
    );

    Ok(())
}

// 辅助函数 - 生成订单ID
pub fn generate_order_id(user_pubkey: &Pubkey, client_order_id: u64, slot: u64) -> u128 {
    let mut hasher = blake3::Hasher::new();
    hasher.update(user_pubkey.as_ref());
    hasher.update(&client_order_id.to_le_bytes());
    hasher.update(&slot.to_le_bytes());
    let hash = hasher.finalize();

    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&hash.as_bytes()[0..16]);
    u128::from_le_bytes(bytes)
}

// 辅助函数 - 检查订单是否存在于OpenOrders中
fn check_order_exists(open_orders: &OpenOrders, order_id: u128) -> bool {
    // 基于位图快速检查订单是否存在
    let bitmap_index = (order_id % 256) as usize / 64;
    let bit_position = (order_id % 64) as u64;

    if bitmap_index < open_orders.orders_bitmap.len() {
        return (open_orders.orders_bitmap[bitmap_index] & (1u64 << bit_position)) != 0;
    }

    false
}

// 辅助函数 - 将订单添加到OpenOrders中
fn add_to_open_orders(open_orders: &mut OpenOrders, order: &Order) -> Result<()> {
    // 检查是否达到最大订单限制
    if open_orders.order_count >= 256 {
        return Err(ErrorCode::OrderBookFull.into());
    }

    // 更新位图
    let bitmap_index = (order.order_id % 256) as usize / 64;
    let bit_position = (order.order_id % 64) as u64;

    if bitmap_index < open_orders.orders_bitmap.len() {
        open_orders.orders_bitmap[bitmap_index] |= 1u64 << bit_position;
        open_orders.order_count += 1;
        return Ok(());
    }

    Err(ErrorCode::InvalidParameters.into())
}

// 辅助函数 - 从OpenOrders中移除订单
fn remove_from_open_orders(open_orders: &mut OpenOrders, order_id: u128) -> Result<()> {
    // 更新位图
    let bitmap_index = (order_id % 256) as usize / 64;
    let bit_position = (order_id % 64) as u64;

    if bitmap_index < open_orders.orders_bitmap.len() {
        open_orders.orders_bitmap[bitmap_index] &= !(1u64 << bit_position);
        open_orders.order_count = open_orders.order_count.saturating_sub(1);
        return Ok(());
    }

    Err(ErrorCode::InvalidParameters.into())
}
