use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, Mint};

declare_id!("AMM11111111111111111111111111111111111111111");

#[program]
pub mod amm {
    use super::*;

    pub fn initialize_pool(
        ctx: Context<InitializePool>,
        fees: PoolFees,
    ) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        pool.token_a_mint = ctx.accounts.token_a_mint.key();
        pool.token_b_mint = ctx.accounts.token_b_mint.key();
        pool.lp_mint = ctx.accounts.lp_mint.key();
        pool.fees = fees;
        pool.authority = ctx.accounts.authority.key();
        Ok(())
    }

    pub fn add_liquidity(
        ctx: Context<AddLiquidity>,
        amount_a: u64,
        amount_b: u64,
        min_lp_tokens: u64,
    ) -> Result<()> {
        // 计算LP代币数量
        let pool = &ctx.accounts.pool;
        let total_supply = ctx.accounts.lp_mint.supply;
        let lp_amount = if total_supply == 0 {
            (amount_a as u128 * amount_b as u128).sqrt() as u64
        } else {
            let ratio = (amount_a as u128 * total_supply as u128) / pool.reserve_a as u128;
            std::cmp::min(
                ratio as u64,
                (amount_b as u128 * total_supply as u128 / pool.reserve_b as u128) as u64,
            )
        };

        require!(lp_amount >= min_lp_tokens, ErrorCode::SlippageExceeded);

        // 转移代币到池子
        token::transfer(
            ctx.accounts.into_transfer_a_context(),
            amount_a,
        )?;
        token::transfer(
            ctx.accounts.into_transfer_b_context(),
            amount_b,
        )?;

        // 铸造LP代币
        token::mint_to(
            ctx.accounts.into_mint_lp_context(),
            lp_amount,
        )?;

        // 更新储备金
        let pool = &mut ctx.accounts.pool;
        pool.reserve_a = pool.reserve_a.checked_add(amount_a).unwrap();
        pool.reserve_b = pool.reserve_b.checked_add(amount_b).unwrap();

        Ok(())
    }

    pub fn remove_liquidity(
        ctx: Context<RemoveLiquidity>,
        lp_amount: u64,
        min_amount_a: u64,
        min_amount_b: u64,
    ) -> Result<()> {
        let pool = &ctx.accounts.pool;
        let total_supply = ctx.accounts.lp_mint.supply;

        // 计算返还的代币数量
        let amount_a = (lp_amount as u128 * pool.reserve_a as u128 / total_supply as u128) as u64;
        let amount_b = (lp_amount as u128 * pool.reserve_b as u128 / total_supply as u128) as u64;

        require!(
            amount_a >= min_amount_a && amount_b >= min_amount_b,
            ErrorCode::SlippageExceeded
        );

        // 销毁LP代币
        token::burn(
            ctx.accounts.into_burn_lp_context(),
            lp_amount,
        )?;

        // 转移代币给用户
        token::transfer(
            ctx.accounts.into_transfer_a_context(),
            amount_a,
        )?;
        token::transfer(
            ctx.accounts.into_transfer_b_context(),
            amount_b,
        )?;

        // 更新储备金
        let pool = &mut ctx.accounts.pool;
        pool.reserve_a = pool.reserve_a.checked_sub(amount_a).unwrap();
        pool.reserve_b = pool.reserve_b.checked_sub(amount_b).unwrap();

        Ok(())
    }

    pub fn swap(
        ctx: Context<Swap>,
        amount_in: u64,
        minimum_amount_out: u64,
    ) -> Result<()> {
        let pool = &ctx.accounts.pool;
        
        // 计算输出金额（使用恒定乘积公式）
        let amount_out = calculate_output_amount(
            amount_in,
            pool.reserve_a,
            pool.reserve_b,
            pool.fees.swap_fee_numerator,
            pool.fees.swap_fee_denominator,
        )?;

        require!(amount_out >= minimum_amount_out, ErrorCode::SlippageExceeded);

        // 执行代币交换
        token::transfer(
            ctx.accounts.into_transfer_in_context(),
            amount_in,
        )?;
        token::transfer(
            ctx.accounts.into_transfer_out_context(),
            amount_out,
        )?;

        // 更新储备金
        let pool = &mut ctx.accounts.pool;
        pool.reserve_a = pool.reserve_a.checked_add(amount_in).unwrap();
        pool.reserve_b = pool.reserve_b.checked_sub(amount_out).unwrap();

        Ok(())
    }
}

#[account]
pub struct Pool {
    pub token_a_mint: Pubkey,
    pub token_b_mint: Pubkey,
    pub lp_mint: Pubkey,
    pub reserve_a: u64,
    pub reserve_b: u64,
    pub fees: PoolFees,
    pub authority: Pubkey,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub struct PoolFees {
    pub swap_fee_numerator: u64,
    pub swap_fee_denominator: u64,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Slippage tolerance exceeded")]
    SlippageExceeded,
    #[msg("Invalid pool tokens")]
    InvalidPoolTokens,
}

#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(init, payer = authority, space = 8 + std::mem::size_of::<Pool>())]
    pub pool: Account<'info, Pool>,
    pub token_a_mint: Account<'info, Mint>,
    pub token_b_mint: Account<'info, Mint>,
    pub lp_mint: Account<'info, Mint>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AddLiquidity<'info> {
    #[account(mut)]
    pub pool: Account<'info, Pool>,
    #[account(mut)]
    pub user_token_a: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_token_b: Account<'info, TokenAccount>,
    #[account(mut)]
    pub pool_token_a: Account<'info, TokenAccount>,
    #[account(mut)]
    pub pool_token_b: Account<'info, TokenAccount>,
    #[account(mut)]
    pub lp_mint: Account<'info, Mint>,
    #[account(mut)]
    pub user_lp: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    #[account(signer)]
    pub authority: Signer<'info>,
}