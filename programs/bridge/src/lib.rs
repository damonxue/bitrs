use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

declare_id!("BRDG111111111111111111111111111111111111111");

#[program]
pub mod bridge {
    use super::*;

    pub fn initialize_bridge(
        ctx: Context<InitializeBridge>,
        validators: Vec<Pubkey>,
        threshold: u64,
    ) -> Result<()> {
        require!(
            threshold as usize <= validators.len(),
            ErrorCode::InvalidThreshold
        );

        let bridge = &mut ctx.accounts.bridge;
        bridge.authority = ctx.accounts.authority.key();
        bridge.validators = validators;
        bridge.threshold = threshold;
        bridge.nonce = 0;
        Ok(())
    }

    pub fn lock_tokens(
        ctx: Context<LockTokens>,
        amount: u64,
        recipient: String,
        target_chain: String,
    ) -> Result<()> {
        let bridge = &mut ctx.accounts.bridge;
        
        // Generate unique transfer ID
        let transfer_id = generate_transfer_id(
            &ctx.accounts.user_token_account.key(),
            &recipient,
            amount,
            bridge.nonce,
        );
        bridge.nonce += 1;

        // Lock tokens in bridge vault
        token::transfer(
            ctx.accounts.into_transfer_context(),
            amount,
        )?;

        // Emit event for relayers
        emit!(TokensLocked {
            transfer_id,
            token: ctx.accounts.token_mint.key(),
            amount,
            sender: ctx.accounts.authority.key(),
            recipient,
            target_chain,
        });

        Ok(())
    }

    pub fn release_tokens(
        ctx: Context<ReleaseTokens>,
        amount: u64,
        transfer_id: [u8; 32],
        signatures: Vec<[u8; 64]>,
    ) -> Result<()> {
        let bridge = &ctx.accounts.bridge;

        // Verify signatures meet threshold
        let valid_sigs = verify_signatures(
            &bridge.validators,
            &signatures,
            &transfer_id,
        )?;

        require!(
            valid_sigs as u64 >= bridge.threshold,
            ErrorCode::InsufficientSignatures
        );

        // Release tokens from bridge vault
        token::transfer(
            ctx.accounts.into_transfer_context(),
            amount,
        )?;

        emit!(TokensReleased {
            transfer_id,
            token: ctx.accounts.token_mint.key(),
            amount,
            recipient: ctx.accounts.recipient_token_account.key(),
        });

        Ok(())
    }
}

#[account]
pub struct Bridge {
    pub authority: Pubkey,
    pub validators: Vec<Pubkey>,
    pub threshold: u64,
    pub nonce: u64,
}

#[derive(Accounts)]
pub struct InitializeBridge<'info> {
    #[account(init, payer = authority, space = 8 + std::mem::size_of::<Bridge>())]
    pub bridge: Account<'info, Bridge>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct LockTokens<'info> {
    #[account(mut)]
    pub bridge: Account<'info, Bridge>,
    pub token_mint: Account<'info, Mint>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub bridge_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    #[account(signer)]
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct ReleaseTokens<'info> {
    #[account(mut)]
    pub bridge: Account<'info, Bridge>,
    pub token_mint: Account<'info, Mint>,
    #[account(mut)]
    pub bridge_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub recipient_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    #[account(signer)]
    pub authority: Signer<'info>,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid threshold")]
    InvalidThreshold,
    #[msg("Insufficient valid signatures")]
    InsufficientSignatures,
}

#[event]
pub struct TokensLocked {
    pub transfer_id: [u8; 32],
    pub token: Pubkey,
    pub amount: u64,
    pub sender: Pubkey,
    pub recipient: String,
    pub target_chain: String,
}

#[event]
pub struct TokensReleased {
    pub transfer_id: [u8; 32],
    pub token: Pubkey,
    pub amount: u64,
    pub recipient: Pubkey,
}

fn generate_transfer_id(
    sender: &Pubkey,
    recipient: &str,
    amount: u64,
    nonce: u64,
) -> [u8; 32] {
    let mut hasher = DefaultHasher::new();
    sender.hash(&mut hasher);
    recipient.hash(&mut hasher);
    amount.hash(&mut hasher);
    nonce.hash(&mut hasher);
    let hash = hasher.finish();
    let mut bytes = [0u8; 32];
    bytes[..8].copy_from_slice(&hash.to_le_bytes());
    bytes
}

fn verify_signatures(
    validators: &[Pubkey],
    signatures: &[[u8; 64]],
    message: &[u8; 32],
) -> Result<usize> {
    let mut valid_count = 0;
    for sig in signatures {
        if validators.iter().any(|v| {
            // In real implementation, verify ed25519 signature
            true // Placeholder
        }) {
            valid_count += 1;
        }
    }
    Ok(valid_count)
}