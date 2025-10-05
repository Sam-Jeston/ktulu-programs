use crate::{
    dlmm::{self},
    ensure_signer_is_owner_or_operator,
    events::{close_position::ClosePositionEvent, create_position::CreatePositionEvent},
    DlmmVaultAccount, VaultErrorCode,
};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct DlmmClosePosition<'info> {
    #[account(
        mut,
        seeds = [b"dlmm_vault".as_ref(), vault_account.owner.as_ref(), vault_account.dlmm_pool_id.as_ref()],
        bump
    )]
    pub vault_account: Account<'info, DlmmVaultAccount>,

    #[account(mut)]
    /// CHECK: The position account to be closed
    pub position: UncheckedAccount<'info>,

    /// CHECK: DLMM program event authority for event CPI
    pub event_authority: UncheckedAccount<'info>,

    #[account(address = dlmm::ID)]
    /// CHECK: DLMM program
    pub dlmm_program: UncheckedAccount<'info>,

    pub signer: Signer<'info>,
}

pub fn handle_dlmm_close_position<'a, 'b, 'c, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, DlmmClosePosition<'info>>,
) -> Result<()> {
    // We can only close a position if the vault is in a position
    if !ctx.accounts.vault_account.in_position {
        return Err(error!(VaultErrorCode::PositionNotOpen));
    }

    // Position closing is valid for both the owner and the operator
    ensure_signer_is_owner_or_operator(&ctx.accounts.signer.key, &ctx.accounts.vault_account)?;

    let accounts = dlmm::cpi::accounts::ClosePosition2 {
        sender: ctx.accounts.vault_account.to_account_info(),
        rent_receiver: ctx.accounts.signer.to_account_info(),
        position: ctx.accounts.position.to_account_info(),
        event_authority: ctx.accounts.event_authority.to_account_info(),
        program: ctx.accounts.dlmm_program.to_account_info(),
    };

    let signer_seeds: &[&[&[u8]]] = &[&[
        b"dlmm_vault",
        ctx.accounts.vault_account.owner.as_ref(),
        ctx.accounts.vault_account.dlmm_pool_id.as_ref(),
        &[ctx.bumps.vault_account],
    ]];

    let cpi_context = CpiContext::new(ctx.accounts.dlmm_program.to_account_info(), accounts)
        .with_signer(signer_seeds);

    dlmm::cpi::close_position2(cpi_context)?;

    ctx.accounts.vault_account.in_position = false;
    ctx.accounts.vault_account.position_id = Pubkey::default();

    emit!(ClosePositionEvent {
        vault_account: ctx.accounts.vault_account.key(),
        position: ctx.accounts.position.key(),
        signer: ctx.accounts.signer.key(),
    });

    Ok(())
}
