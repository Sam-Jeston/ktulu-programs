use anchor_lang::prelude::*;
use anchor_spl::token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked};

use crate::{
    ensure_signer_is_owner, events::deposit::DepositEvent, DlmmVaultAccount, VaultErrorCode,
};

#[derive(Accounts)]
pub struct DlmmDeposit<'info> {
    pub vault_account: Account<'info, DlmmVaultAccount>,
    #[account(mut)]
    pub signer: Signer<'info>,

    pub token_x_mint: InterfaceAccount<'info, Mint>,
    pub vault_owner_token_x: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = token_x_mint,
        associated_token::authority = vault_account,
        associated_token::token_program = token_x_program
    )]
    pub vault_token_x_account: InterfaceAccount<'info, TokenAccount>,
    pub token_x_program: Interface<'info, TokenInterface>,

    pub token_y_mint: InterfaceAccount<'info, Mint>,
    #[account(mut)]
    pub vault_owner_token_y: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = token_y_mint,
        associated_token::authority = vault_account,
        associated_token::token_program = token_y_program
    )]
    pub vault_token_y_account: InterfaceAccount<'info, TokenAccount>,
    pub token_y_program: Interface<'info, TokenInterface>,
}

pub fn handle_deposit<'a, 'b, 'c, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, DlmmDeposit<'info>>,
    token_x_deposit_amount: u64,
    token_y_deposit_amount: u64,
) -> Result<()> {
    // Access to deposit is limitted to the owner of the vault
    ensure_signer_is_owner(&ctx.accounts.signer.key, &ctx.accounts.vault_account)?;

    // At least one deposit amount must be greater than 0
    if token_x_deposit_amount == 0 && token_y_deposit_amount == 0 {
        return Err(error!(VaultErrorCode::InvalidDepositAmount));
    }

    // Transfer from the vault_owner_token_x account to the vault_account.token_x_ata account
    token_interface::transfer_checked(
        CpiContext::new(
            ctx.accounts.token_x_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.vault_owner_token_x.to_account_info(),
                to: ctx.accounts.vault_token_x_account.to_account_info(),
                authority: ctx.accounts.signer.to_account_info(),
                mint: ctx.accounts.token_x_mint.to_account_info(),
            },
        ),
        token_x_deposit_amount,
        ctx.accounts.token_x_mint.decimals,
    )?;

    // Transfer from the vault_owner_token_y account to the vault_account.token_y_ata account
    token_interface::transfer_checked(
        CpiContext::new(
            ctx.accounts.token_y_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.vault_owner_token_y.to_account_info(),
                to: ctx.accounts.vault_token_y_account.to_account_info(),
                authority: ctx.accounts.signer.to_account_info(),
                mint: ctx.accounts.token_y_mint.to_account_info(),
            },
        ),
        token_y_deposit_amount,
        ctx.accounts.token_y_mint.decimals,
    )?;

    emit!(DepositEvent {
        vault_account: ctx.accounts.vault_account.key(),
        token_x_deposit_amount,
        token_y_deposit_amount,
    });

    Ok(())
}
