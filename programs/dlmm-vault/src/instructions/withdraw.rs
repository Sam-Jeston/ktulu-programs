use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, TransferChecked};

use crate::{events::withdraw::WithdrawEvent, DlmmVaultAccount, VaultErrorCode};

#[derive(Accounts)]
pub struct DlmmWithdraw<'info> {
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

pub fn handle_dlmm_withdraw<'a, 'b, 'c, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, DlmmWithdraw<'info>>,
    token_x_withdraw_amount: u64,
    token_y_withdraw_amount: u64,
) -> Result<()> {
    // Access to withdraw is limitted to the owner of the vault
    if ctx.accounts.signer.key() != ctx.accounts.vault_account.owner {
        return Err(error!(VaultErrorCode::InvalidSigner));
    }

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

    // NOTE: Withdraw amounts can be zero, only withdraw where balance is > 0
    if token_x_withdraw_amount > ctx.accounts.vault_token_x_account.amount {
        return Err(error!(VaultErrorCode::InvalidWithdrawAmount));
    }
    if token_y_withdraw_amount > ctx.accounts.vault_token_y_account.amount {
        return Err(error!(VaultErrorCode::InvalidWithdrawAmount));
    }

    let (expect, bump) = Pubkey::find_program_address(
        &[
            b"dlmm_vault",
            ctx.accounts.signer.key.as_ref(),
            ctx.accounts.vault_account.dlmm_pool_id.as_ref(),
        ],
        ctx.program_id,
    );
    require_keys_eq!(expect, ctx.accounts.vault_account.key());

    if token_x_withdraw_amount > 0 {
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"dlmm_vault",
            ctx.accounts.signer.key.as_ref(),
            ctx.accounts.vault_account.dlmm_pool_id.as_ref(),
            &[bump.clone()],
        ]];
        token::transfer_checked(
            CpiContext::new(
                ctx.accounts.token_x_program.to_account_info(),
                TransferChecked {
                    from: ctx.accounts.vault_token_x_account.to_account_info(),
                    to: ctx.accounts.vault_owner_token_x.to_account_info(),
                    authority: ctx.accounts.vault_account.to_account_info(),
                    mint: ctx.accounts.token_x_mint.to_account_info(),
                },
            )
            .with_signer(signer_seeds),
            token_x_withdraw_amount,
            ctx.accounts.token_x_mint.decimals,
        )?;
    }

    if token_y_withdraw_amount > 0 {
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"dlmm_vault",
            ctx.accounts.signer.key.as_ref(),
            ctx.accounts.vault_account.dlmm_pool_id.as_ref(),
            &[bump],
        ]];
        token::transfer_checked(
            CpiContext::new(
                ctx.accounts.token_y_program.to_account_info(),
                TransferChecked {
                    from: ctx.accounts.vault_token_y_account.to_account_info(),
                    to: ctx.accounts.vault_owner_token_y.to_account_info(),
                    authority: ctx.accounts.vault_account.to_account_info(),
                    mint: ctx.accounts.token_y_mint.to_account_info(),
                },
            )
            .with_signer(signer_seeds),
            token_y_withdraw_amount,
            ctx.accounts.token_y_mint.decimals,
        )?;
    }

    emit!(WithdrawEvent {
        vault_account: ctx.accounts.vault_account.key(),
        token_x_withdraw_amount,
        token_y_withdraw_amount,
    });

    Ok(())
}
