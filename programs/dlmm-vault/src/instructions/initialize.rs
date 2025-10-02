use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

use crate::{events::initialize::InitializeEvent, DlmmVaultAccount};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = signer,
        space = 8 + DlmmVaultAccount::INIT_SPACE,
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
        init,
        payer = signer,
        associated_token::mint = token_x_mint,
        associated_token::authority = vault_account,
        associated_token::token_program = token_x_program
    )]
    pub token_x_ata: Account<'info, TokenAccount>,

    pub token_y_mint: Account<'info, Mint>,
    pub token_y_program: Program<'info, Token>,
    #[account(
        init,
        payer = signer,
        associated_token::mint = token_y_mint,
        associated_token::authority = vault_account,
        associated_token::token_program = token_y_program
    )]
    pub token_y_ata: Account<'info, TokenAccount>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn handle_initialize<'a, 'b, 'c, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, Initialize<'info>>,
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

    emit!(InitializeEvent {
        vault_account: ctx.accounts.vault_account.key(),
        owner: ctx.accounts.signer.key(),
        token_x_mint: token_x_mint,
        token_y_mint: token_y_mint,
        dlmm_pool: ctx.accounts.dlmm_pool.key(),
        lower_price_range_bps: lower_price_range_bps,
        upper_price_range_bps: upper_price_range_bps,
        operator: operator,
        position_id: Pubkey::default(),
    });

    Ok(())
}
