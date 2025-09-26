use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::{close_account, CloseAccount},
};

use crate::{ensure_signer_is_owner_or_operator, DlmmVaultAccount};
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

#[derive(Accounts)]
pub struct InitializeHarvestAta<'info> {
    #[account(
        mut,
        seeds = [b"dlmm_vault".as_ref(), vault_account.owner.as_ref(), vault_account.dlmm_pool_id.as_ref()],
        bump
    )]
    pub vault_account: Account<'info, DlmmVaultAccount>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,

    pub harvest_mint: InterfaceAccount<'info, Mint>,
    pub harvest_program: Interface<'info, TokenInterface>,
    #[account(
        init,
        payer = signer,
        associated_token::mint = harvest_mint,
        associated_token::authority = vault_account,
        associated_token::token_program = harvest_program
    )]
    pub harvest_ata: InterfaceAccount<'info, TokenAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
pub struct CloseHarvestAta<'info> {
    #[account(
        mut,
        seeds = [b"dlmm_vault".as_ref(), vault_account.owner.as_ref(), vault_account.dlmm_pool_id.as_ref()],
        bump
    )]
    pub vault_account: Account<'info, DlmmVaultAccount>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,

    pub harvest_mint: InterfaceAccount<'info, Mint>,
    pub harvest_program: Interface<'info, TokenInterface>,
    #[account(
        mut,
        associated_token::mint = harvest_mint,
        associated_token::authority = vault_account,
        associated_token::token_program = harvest_program
    )]
    pub harvest_ata: InterfaceAccount<'info, TokenAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn handle_initialize_harvest_ata<'a, 'b, 'c, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, InitializeHarvestAta<'info>>,
) -> Result<()> {
    ensure_signer_is_owner_or_operator(&ctx.accounts.signer.key, &ctx.accounts.vault_account)?;
    Ok(())
}

pub fn handle_close_harvest_ata<'a, 'b, 'c, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, CloseHarvestAta<'info>>,
) -> Result<()> {
    ensure_signer_is_owner_or_operator(&ctx.accounts.signer.key, &ctx.accounts.vault_account)?;

    // PDA signer seeds
    let seeds = &[
        b"dlmm_vault".as_ref(),
        ctx.accounts.vault_account.owner.as_ref(),
        ctx.accounts.vault_account.dlmm_pool_id.as_ref(),
        &[ctx.bumps.vault_account],
    ];
    let signer = &[&seeds[..]];

    let cpi = CpiContext::new(
        ctx.accounts.harvest_program.to_account_info(),
        CloseAccount {
            account: ctx.accounts.harvest_ata.to_account_info(),
            destination: ctx.accounts.signer.to_account_info(),
            authority: ctx.accounts.vault_account.to_account_info(),
        },
    )
    .with_signer(signer);

    close_account(cpi)?;

    Ok(())
}
