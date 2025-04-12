use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use std::cmp;

// 声明子模块
pub mod advanced_orders;
pub mod core;
pub mod cross_chain;
pub mod events;
pub mod lp_mining;
pub mod orderbook;
pub mod risk;
pub mod storage;

// 重新导出主要类型，方便使用
pub use advanced_orders::{AdvancedOrderType, TradingStrategy};
pub use core::{Market, OpenOrders};
pub use cross_chain::{ChainId, CrossChainBridge};
pub use events::EventHandler;
pub use lp_mining::{LpPool, UserStake};
pub use orderbook::{Order, OrderBook, OrderType, Side};
pub use risk::RiskEngine;
pub use storage::OptimizedStorage;

declare_id!("DEX1111111111111111111111111111111111111111");

#[program]
pub mod dex_core {
    use super::*;

    // 初始化市场
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
        core::initialize_market(
            ctx,
            market_name,
            base_decimals,
            quote_decimals,
            lot_size,
            tick_size,
            maker_fee,
            taker_fee,
        )
    }

    // 下单
    pub fn place_order(
        ctx: Context<PlaceOrder>,
        side: Side,
        limit_price: u64,
        max_quantity: u64,
        order_type: OrderType,
        self_trade_behavior: SelfTradeBehavior,
        client_order_id: u64,
    ) -> Result<()> {
        core::place_order(
            ctx,
            side,
            limit_price,
            max_quantity,
            order_type,
            self_trade_behavior,
            client_order_id,
        )
    }

    // 取消订单
    pub fn cancel_order(ctx: Context<CancelOrder>, order_id: u128, side: Side) -> Result<()> {
        core::cancel_order(ctx, order_id, side)
    }

    // 结算资金
    pub fn settle_funds(ctx: Context<SettleFunds>) -> Result<()> {
        core::settle_funds(ctx)
    }

    // 添加流动性
    pub fn add_liquidity(
        ctx: Context<AddLiquidity>,
        base_amount: u64,
        quote_amount: u64,
    ) -> Result<()> {
        lp_mining::add_liquidity(ctx, base_amount, quote_amount)
    }

    // 移除流动性
    pub fn remove_liquidity(ctx: Context<RemoveLiquidity>, lp_amount: u64) -> Result<()> {
        lp_mining::remove_liquidity(ctx, lp_amount)
    }

    // 领取流动性挖矿奖励
    pub fn claim_lp_rewards(ctx: Context<ClaimRewards>) -> Result<()> {
        lp_mining::claim_rewards(ctx)
    }

    // 创建高级订单
    pub fn create_advanced_order(
        ctx: Context<CreateAdvancedOrder>,
        advanced_order_type: AdvancedOrderType,
        params: Vec<u8>, // 序列化的参数
    ) -> Result<()> {
        advanced_orders::create_advanced_order(ctx, advanced_order_type, params)
    }

    // 创建跨链订单
    pub fn create_cross_chain_order(
        ctx: Context<CreateCrossChainOrder>,
        target_chain_id: ChainId,
        order_data: Vec<u8>, // 序列化的订单数据
    ) -> Result<()> {
        cross_chain::create_cross_chain_order(ctx, target_chain_id, order_data)
    }

    // 确认跨链交易
    pub fn confirm_cross_chain_transaction(
        ctx: Context<ConfirmCrossChainTx>,
        source_chain_id: ChainId,
        tx_hash: [u8; 32],
        proof_data: Vec<u8>,
    ) -> Result<()> {
        cross_chain::confirm_transaction(ctx, source_chain_id, tx_hash, proof_data)
    }
}

// 订单类型和自成交行为等定义已经移动到相应的模块中
pub use core::SelfTradeBehavior;

// 统一错误码定义
#[error_code]
pub enum ErrorCode {
    #[msg("市场未激活")]
    MarketNotActive,
    #[msg("无效的订单数量")]
    InvalidOrderQuantity,
    #[msg("无效的订单价格")]
    InvalidOrderPrice,
    #[msg("订单未找到")]
    OrderNotFound,
    #[msg("无权操作")]
    UnauthorizedOperation,
    #[msg("余额不足")]
    InsufficientFunds,
    #[msg("自成交")]
    SelfTrade,
    #[msg("没有可结算的资金")]
    NoFundsToSettle,
    #[msg("订单簿已满")]
    OrderBookFull,
    #[msg("无效的市场ID")]
    InvalidMarketId,
    #[msg("无效的用户账户")]
    InvalidUserAccount,
    #[msg("参数错误")]
    InvalidParameters,
    #[msg("流动性不足")]
    InsufficientLiquidity,
    #[msg("LP代币不足")]
    InsufficientLpTokens,
    #[msg("无效的跨链参数")]
    InvalidCrossChainParams,
    #[msg("无效的高级订单参数")]
    InvalidAdvancedOrderParams,
    #[msg("不支持的链ID")]
    UnsupportedChainId,
    #[msg("无效的证明数据")]
    InvalidProofData,
    #[msg("存储已满")]
    StorageFull,
}
