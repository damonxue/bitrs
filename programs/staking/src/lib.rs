use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint};

declare_id!("STAKE111111111111111111111111111111111111111");

#[program]
pub mod staking {
    use super::*;

    pub fn initialize_staking_pool(
        ctx: Context<InitializeStakingPool>,
        reward_duration: u64,
        reward_rate: u64,
    ) -> Result<()> {
        let pool = &mut ctx.accounts.staking_pool;
        pool.authority = ctx.accounts.authority.key();
        pool.stake_mint = ctx.accounts.stake_mint.key();
        pool.reward_mint = ctx.accounts.reward_mint.key();
        pool.reward_duration = reward_duration;
        pool.reward_rate = reward_rate;
        pool.last_update_time = Clock::get()?.unix_timestamp as u64;
        Ok(())
    }

    pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
        let pool = &mut ctx.accounts.staking_pool;
        let user_stake = &mut ctx.accounts.user_stake;

        // Update rewards
        update_rewards(pool, user_stake)?;

        // Transfer tokens to stake
        token::transfer(
            ctx.accounts.into_transfer_context(),
            amount,
        )?;

        // Update user stake
        user_stake.staked_amount = user_stake.staked_amount.checked_add(amount)
            .ok_or(ErrorCode::NumericOverflow)?;
        
        Ok(())
    }

    pub fn unstake(ctx: Context<Unstake>, amount: u64) -> Result<()> {
        let pool = &mut ctx.accounts.staking_pool;
        let user_stake = &mut ctx.accounts.user_stake;

        // Update and claim rewards first
        update_rewards(pool, user_stake)?;
        claim_rewards(ctx.accounts.into_claim_context())?;

        require!(
            user_stake.staked_amount >= amount,
            ErrorCode::InsufficientStakedAmount
        );

        // Transfer staked tokens back
        token::transfer(
            ctx.accounts.into_transfer_context(),
            amount,
        )?;

        // Update user stake
        user_stake.staked_amount = user_stake.staked_amount.checked_sub(amount)
            .ok_or(ErrorCode::NumericOverflow)?;

        Ok(())
    }

    pub fn claim_reward(ctx: Context<ClaimReward>) -> Result<()> {
        let pool = &mut ctx.accounts.staking_pool;
        let user_stake = &mut ctx.accounts.user_stake;

        // Update rewards
        update_rewards(pool, user_stake)?;

        // Transfer rewards
        claim_rewards(ctx.accounts.into_claim_context())?;

        Ok(())
    }
}

#[account]
pub struct StakingPool {
    pub authority: Pubkey,
    pub stake_mint: Pubkey,
    pub reward_mint: Pubkey,
    pub total_staked: u64,
    pub reward_rate: u64,
    pub reward_duration: u64,
    pub last_update_time: u64,
    pub reward_per_token_stored: u128,
}

#[account]
pub struct UserStake {
    pub owner: Pubkey,
    pub pool: Pubkey,
    pub staked_amount: u64,
    pub rewards_earned: u64,
    pub reward_per_token_paid: u128,
}

#[derive(Accounts)]
pub struct InitializeStakingPool<'info> {
    #[account(init, payer = authority, space = 8 + std::mem::size_of::<StakingPool>())]
    pub staking_pool: Account<'info, StakingPool>,
    pub stake_mint: Account<'info, Mint>,
    pub reward_mint: Account<'info, Mint>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub staking_pool: Account<'info, StakingPool>,
    #[account(
        mut,
        constraint = user_stake.owner == authority.key(),
        constraint = user_stake.pool == staking_pool.key(),
    )]
    pub user_stake: Account<'info, UserStake>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub pool_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    #[account(signer)]
    pub authority: Signer<'info>,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Numeric overflow")]
    NumericOverflow,
    #[msg("Insufficient staked amount")]
    InsufficientStakedAmount,
}