// https://github.com/coral-xyz/anchor/issues/3401#issuecomment-2513466441
#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;

pub mod instructions;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};
pub use instructions::*;

declare_program!(dlmm);

use crate::instructions::swap::*;

declare_id!("4JTNRRQpgLusbEhGnzTuE9kgPgMLXQX1wqBzU52GduqH");

#[program]
pub mod dlmm_vault {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        token_x_mint: Pubkey,
        token_y_mint: Pubkey,
        lower_price_range_bps: u64,
        upper_price_range_bps: u64,
        operator: Pubkey,
    ) -> Result<()> {
        ctx.accounts.vault_account.dlmm_pool_id = ctx.accounts.dlmm_pool.key();
        ctx.accounts.vault_account.token_x_mint = token_x_mint;
        ctx.accounts.vault_account.token_y_mint = token_y_mint;
        ctx.accounts.vault_account.lower_price_range_bps = lower_price_range_bps;
        ctx.accounts.vault_account.upper_price_range_bps = upper_price_range_bps;
        ctx.accounts.vault_account.owner = ctx.accounts.signer.key();
        ctx.accounts.vault_account.operator = operator;
        ctx.accounts.vault_account.in_position = false;
        ctx.accounts.vault_account.position_id = Pubkey::default();
        Ok(())
    }

    pub fn dlmm_swap<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, DlmmSwap<'info>>,
        amount_in: u64,
        min_amount_out: u64,
    ) -> Result<()> {
        instructions::swap::handle_dlmm_swap(ctx, amount_in, min_amount_out)
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = signer,
        space = 8 + 32 + 32 + 32 + 32 + 32 + 8 + 8,
        seeds = [b"dlmm_vault".as_ref(), signer.key().as_ref(), dlmm_pool.key.as_ref()],
        bump
    )]
    pub vault_account: Account<'info, DlmmVaultAccount>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub dlmm_pool: UncheckedAccount<'info>,

    pub token_x_mint: Account<'info, Mint>,
    pub token_x_program: Program<'info, Token>,
    #[account(
        init_if_needed,
        payer = signer,
        token::mint = token_x_mint,
        token::authority = vault_account,
        token::token_program = token_x_program
    )]
    pub token_x_ata: Account<'info, TokenAccount>,

    pub token_y_mint: Account<'info, Mint>,
    pub token_y_program: Program<'info, Token>,
    #[account(
        init_if_needed,
        payer = signer,
        token::mint = token_y_mint,
        token::authority = vault_account,
        token::token_program = token_y_program
    )]
    pub token_y_ata: Account<'info, TokenAccount>,
}

#[account]
pub struct DlmmVaultAccount {
    // This owner will lets us effectively memcmp filter for vault accounts per user
    owner: Pubkey,
    in_position: bool,
    operator: Pubkey,
    dlmm_pool_id: Pubkey,
    position_id: Pubkey,
    token_x_mint: Pubkey,
    token_y_mint: Pubkey,
    lower_price_range_bps: u64,
    upper_price_range_bps: u64,
}
