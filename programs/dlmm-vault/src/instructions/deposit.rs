use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, TransferChecked};

use crate::{DlmmVaultAccount, VaultErrorCode};

#[derive(Accounts)]
pub struct DlmmDeposit<'info> {
    pub vault_account: Account<'info, DlmmVaultAccount>,
    #[account(mut)]
    pub signer: Signer<'info>,

    pub token_x_mint: Account<'info, Mint>,
    #[account(mut)]
    pub vault_owner_token_x: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault_token_x_account: Account<'info, TokenAccount>,
    pub token_x_program: Program<'info, Token>,

    pub token_y_mint: Account<'info, Mint>,
    #[account(mut)]
    pub vault_owner_token_y: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault_token_y_account: Account<'info, TokenAccount>,
    pub token_y_program: Program<'info, Token>,
}

pub fn handle_dlmm_deposit<'a, 'b, 'c, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, DlmmDeposit<'info>>,
    token_x_deposit_amount: u64,
    token_y_deposit_amount: u64,
) -> Result<()> {
    // Validate that the vault_owner_token_x account is an ATA for vault_account.token_x_mint
    if ctx.accounts.vault_owner_token_x.mint != ctx.accounts.vault_account.token_x_mint {
        return Err(error!(VaultErrorCode::InvalidTokenAccount));
    }

    // Validate that the vault_owner_token_y account is an ATA for vault_account.token_y_mint
    if ctx.accounts.vault_owner_token_y.mint != ctx.accounts.vault_account.token_y_mint {
        return Err(error!(VaultErrorCode::InvalidTokenAccount));
    }

    // Validate that the owner of the vault is the owner of the token accounts
    if ctx.accounts.vault_owner_token_x.owner != ctx.accounts.vault_account.owner {
        return Err(error!(VaultErrorCode::InvalidTokenAccount));
    }

    if ctx.accounts.vault_owner_token_y.owner != ctx.accounts.vault_account.owner {
        return Err(error!(VaultErrorCode::InvalidTokenAccount));
    }

    // Deposit amounts must be greater than 0
    if token_x_deposit_amount == 0 || token_y_deposit_amount == 0 {
        return Err(error!(VaultErrorCode::InvalidDepositAmount));
    }

    // Transfer from the vault_owner_token_x account to the vault_account.token_x_ata account
    token::transfer_checked(
        CpiContext::new(
            ctx.accounts.token_x_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.vault_owner_token_x.to_account_info(),
                to: ctx.accounts.vault_token_x_account.to_account_info(),
                authority: ctx.accounts.signer.to_account_info(),
                mint: ctx.accounts.token_x_mint.to_account_info(),
            },
        )
        .with_remaining_accounts(ctx.remaining_accounts.to_vec()),
        token_x_deposit_amount,
        ctx.accounts.token_x_mint.decimals,
    )?;

    // Transfer from the vault_owner_token_y account to the vault_account.token_y_ata account
    token::transfer_checked(
        CpiContext::new(
            ctx.accounts.token_y_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.vault_owner_token_y.to_account_info(),
                to: ctx.accounts.vault_token_y_account.to_account_info(),
                authority: ctx.accounts.signer.to_account_info(),
                mint: ctx.accounts.token_y_mint.to_account_info(),
            },
        )
        .with_remaining_accounts(ctx.remaining_accounts.to_vec()),
        token_y_deposit_amount,
        ctx.accounts.token_y_mint.decimals,
    )?;

    Ok(())
}
