// Closes the associated ATAs and Vault account
// Validates that vault_token_x and vault_token_y balances are zero
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{
    close_account, CloseAccount, Mint, TokenAccount, TokenInterface,
};

use crate::close_vault::CloseVaultEvent;
use crate::{DlmmVaultAccount, VaultErrorCode};

#[derive(Accounts)]
pub struct CloseVault<'info> {
    #[account(
        mut,
        seeds = [b"dlmm_vault", owner.key().as_ref(), vault_account.dlmm_pool_id.as_ref()],
        bump,
        close = owner
    )]
    pub vault_account: Account<'info, DlmmVaultAccount>,

    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut)]
    pub vault_token_x: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub vault_token_y: InterfaceAccount<'info, TokenAccount>,

    pub token_x_mint: InterfaceAccount<'info, Mint>,
    pub token_x_program: Interface<'info, TokenInterface>,
    pub token_y_mint: InterfaceAccount<'info, Mint>,
    pub token_y_program: Interface<'info, TokenInterface>,

    // Destination ATAs
    #[account(mut)]
    pub owner_ata_x: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub owner_ata_y: InterfaceAccount<'info, TokenAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handle_close_vault(ctx: Context<CloseVault>) -> Result<()> {
    require!(
        !ctx.accounts.vault_account.in_position,
        VaultErrorCode::PositionStillOpen
    );

    // Ensure the token balances are zero. If they aren't, a withdraw instruction is
    // required first.
    require!(
        ctx.accounts.vault_token_x.amount == 0,
        VaultErrorCode::NonZeroBalance
    );
    require!(
        ctx.accounts.vault_token_y.amount == 0,
        VaultErrorCode::NonZeroBalance
    );

    // The only user who can call `close` is the vault owner. Ensure the signer is the vault owner.
    require!(
        ctx.accounts.owner.key() == ctx.accounts.vault_account.owner,
        VaultErrorCode::InvalidSigner
    );

    // PDA signer seeds
    let seeds = &[
        b"dlmm_vault".as_ref(),
        ctx.accounts.owner.key.as_ref(),
        ctx.accounts.vault_account.dlmm_pool_id.as_ref(),
        &[ctx.bumps.vault_account],
    ];
    let signer = &[&seeds[..]];

    close_token_account(
        &ctx.accounts.vault_token_x,
        &ctx.accounts.owner.to_account_info(), // lamports destination
        &ctx.accounts.vault_account.to_account_info(), // authority
        &ctx.accounts.token_x_program,
        signer,
    )?;
    close_token_account(
        &ctx.accounts.vault_token_y,
        &ctx.accounts.owner.to_account_info(),
        &ctx.accounts.vault_account.to_account_info(),
        &ctx.accounts.token_y_program,
        signer,
    )?;

    emit!(CloseVaultEvent {
        vault_account: ctx.accounts.vault_account.key(),
    });

    Ok(())
}

fn close_token_account<'info>(
    account: &InterfaceAccount<'info, TokenAccount>,
    destination: &AccountInfo<'info>,
    authority: &AccountInfo<'info>,
    token_prog: &Interface<'info, TokenInterface>,
    signer: &[&[&[u8]]],
) -> Result<()> {
    let cpi = CpiContext::new(
        token_prog.to_account_info(),
        CloseAccount {
            account: account.to_account_info(),
            destination: destination.clone(),
            authority: authority.clone(),
        },
    )
    .with_signer(signer);

    close_account(cpi)
}
