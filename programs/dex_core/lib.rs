use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

declare_id!("DEX1111111111111111111111111111111111111111");

#[program]
pub mod dex_core {
    use super::*;

    pub fn initialize_market(
        ctx: Context<InitializeMarket>,
        market_name: String,
        base_decimals: u8,
        quote_decimals: u8,
        lot_size: u64,
        tick_size: u64,
    ) -> Result<()> {
        let market = &mut ctx.accounts.market;
        market.base_mint = ctx.accounts.base_mint.key();
        market.quote_mint = ctx.accounts.quote_mint.key();
        market.lot_size = lot_size;
        market.tick_size = tick_size;
        market.base_decimals = base_decimals;
        market.quote_decimals = quote_decimals;
        market.name = market_name;
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
        // 实现订单提交逻辑
        Ok(())
    }

    pub fn cancel_order(
        ctx: Context<CancelOrder>,
        order_id: u128,
        side: Side,
    ) -> Result<()> {
        // 实现订单取消逻辑
        Ok(())
    }

    pub fn settle_funds(
        ctx: Context<SettleFunds>,
    ) -> Result<()> {
        // 实现资金结算逻辑
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeMarket<'info> {
    #[account(init, payer = authority, space = 8 + Market::LEN)]
    pub market: Account<'info, Market>,
    pub base_mint: Account<'info, token::Mint>,
    pub quote_mint: Account<'info, token::Mint>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct PlaceOrder<'info> {
    #[account(mut)]
    pub market: Account<'info, Market>,
    #[account(mut)]
    pub order_book: AccountLoader<'info, OrderBook>,
    #[account(mut)]
    pub open_orders: Account<'info, OpenOrders>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(signer)]
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct Market {
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub lot_size: u64,
    pub tick_size: u64,
    pub base_decimals: u8,
    pub quote_decimals: u8,
    pub name: String,
}

impl Market {
    pub const LEN: usize = 32 + 32 + 8 + 8 + 1 + 1 + 32;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq)]
pub enum Side {
    Bid,
    Ask,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq)]
pub enum OrderType {
    Limit,
    Market,
    PostOnly,
    ImmediateOrCancel,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq)]
pub enum SelfTradeBehavior {
    DecrementTake,
    CancelProvide,
    AbortTransaction,
}