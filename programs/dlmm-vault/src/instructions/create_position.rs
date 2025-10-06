use crate::{
    dlmm::{self},
    ensure_signer_is_owner_or_operator,
    events::create_position::CreatePositionEvent,
    DlmmVaultAccount, VaultErrorCode,
};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct DlmmCreatePosition<'info> {
    #[account(
        mut,
        seeds = [b"dlmm_vault".as_ref(), vault_account.owner.as_ref(), vault_account.dlmm_pool_id.as_ref()],
        bump
    )]
    pub vault_account: Account<'info, DlmmVaultAccount>,

    #[account(mut)]
    /// CHECK: The pool account
    pub lb_pair: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: The position account to be created
    pub position: UncheckedAccount<'info>,

    /// CHECK: DLMM program event authority for event CPI
    pub event_authority: UncheckedAccount<'info>,

    #[account(address = dlmm::ID)]
    /// CHECK: DLMM program
    pub dlmm_program: UncheckedAccount<'info>,

    pub signer: Signer<'info>,
    pub rent: UncheckedAccount<'info>,
    pub system_program: UncheckedAccount<'info>,
}

pub fn handle_dlmm_create_position<'a, 'b, 'c, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, DlmmCreatePosition<'info>>,
    lower_bin_id: i32,
    width: i32,
) -> Result<()> {
    // We can only create a position if the vault is not already in a position
    if ctx.accounts.vault_account.in_position {
        return Err(error!(VaultErrorCode::PositionStillOpen));
    }

    if width != ctx.accounts.vault_account.bin_width as i32 {
        return Err(error!(VaultErrorCode::InvalidWidth));
    }

    // Position creation is valid for both the owner and the operator
    ensure_signer_is_owner_or_operator(&ctx.accounts.signer.key, &ctx.accounts.vault_account)?;

    let accounts = dlmm::cpi::accounts::InitializePositionPda {
        owner: ctx.accounts.vault_account.to_account_info(),
        payer: ctx.accounts.signer.to_account_info(),
        // TODO: Is this right??
        base: ctx.accounts.vault_account.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
        rent: ctx.accounts.rent.to_account_info(),
        lb_pair: ctx.accounts.lb_pair.to_account_info(),
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

    dlmm::cpi::initialize_position_pda(cpi_context, lower_bin_id, width)?;

    ctx.accounts.vault_account.in_position = true;
    ctx.accounts.vault_account.position_id = ctx.accounts.position.key();

    emit!(CreatePositionEvent {
        vault_account: ctx.accounts.vault_account.key(),
        position: ctx.accounts.position.key(),
        signer: ctx.accounts.signer.key(),
        lower_bin_id,
        width,
    });

    Ok(())
}
