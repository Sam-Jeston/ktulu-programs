use anchor_lang::prelude::*;

use crate::{
    ensure_signer_is_owner, events::withdraw::WithdrawEvent, DlmmVaultAccount, VaultErrorCode,
};
use anchor_spl::token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked};

#[derive(Accounts)]
pub struct DlmmWithdraw<'info> {
    #[account(
        mut,
        seeds = [b"dlmm_vault".as_ref(), vault_account.owner.as_ref(), vault_account.dlmm_pool_id.as_ref()],
        bump
    )]
    pub vault_account: Account<'info, DlmmVaultAccount>,
    #[account(mut)]
    pub signer: Signer<'info>,

    pub token_x_mint: InterfaceAccount<'info, Mint>,
    #[account(mut)]
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

    #[account(mut)]
    pub vault_owner_harvest_token: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        token::mint = harvest_mint,
        token::authority = vault_account,
        token::token_program = harvest_mint_program,
        seeds = [b"harvest".as_ref(), vault_account.key().as_ref()],
        bump,
    )]
    pub harvest_token: InterfaceAccount<'info, TokenAccount>,

    pub harvest_mint: InterfaceAccount<'info, Mint>,
    pub harvest_mint_program: Interface<'info, TokenInterface>,
}

pub fn handle_withdraw<'a, 'b, 'c, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, DlmmWithdraw<'info>>,
    token_x_withdraw_amount: u64,
    token_y_withdraw_amount: u64,
    harvest_token_withdraw_amount: u64,
) -> Result<()> {
    ensure_signer_is_owner(&ctx.accounts.signer.key, &ctx.accounts.vault_account)?;
    // NOTE: Withdraw amounts can be zero, only withdraw where balance is > 0
    if token_x_withdraw_amount > ctx.accounts.vault_token_x_account.amount {
        return Err(error!(VaultErrorCode::InvalidWithdrawAmount));
    }
    if token_y_withdraw_amount > ctx.accounts.vault_token_y_account.amount {
        return Err(error!(VaultErrorCode::InvalidWithdrawAmount));
    }
    if harvest_token_withdraw_amount > ctx.accounts.harvest_token.amount {
        return Err(error!(VaultErrorCode::InvalidWithdrawAmount));
    }

    if token_x_withdraw_amount > 0 {
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"dlmm_vault",
            ctx.accounts.signer.key.as_ref(),
            ctx.accounts.vault_account.dlmm_pool_id.as_ref(),
            &[ctx.bumps.vault_account],
        ]];
        token_interface::transfer_checked(
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
            &[ctx.bumps.vault_account],
        ]];
        token_interface::transfer_checked(
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

    if harvest_token_withdraw_amount > 0 {
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"dlmm_vault",
            ctx.accounts.signer.key.as_ref(),
            ctx.accounts.vault_account.dlmm_pool_id.as_ref(),
            &[ctx.bumps.vault_account],
        ]];
        token_interface::transfer_checked(
            CpiContext::new(
                ctx.accounts.harvest_mint_program.to_account_info(),
                TransferChecked {
                    from: ctx.accounts.harvest_token.to_account_info(),
                    to: ctx.accounts.vault_owner_harvest_token.to_account_info(),
                    authority: ctx.accounts.vault_account.to_account_info(),
                    mint: ctx.accounts.harvest_mint.to_account_info(),
                },
            )
            .with_signer(signer_seeds),
            harvest_token_withdraw_amount,
            ctx.accounts.harvest_mint.decimals,
        )?;
    }

    emit!(WithdrawEvent {
        vault_account: ctx.accounts.vault_account.key(),
        token_x_withdraw_amount,
        token_y_withdraw_amount,
        harvest_token_withdraw_amount,
    });

    Ok(())
}
