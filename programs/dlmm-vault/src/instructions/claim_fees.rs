use crate::{
    dlmm::{self, types::RemainingAccountsInfo},
    events::{claim_fees::ClaimFeesEvent, create_position::CreatePositionEvent},
    DlmmVaultAccount, VaultErrorCode,
};
use anchor_lang::prelude::*;
use anchor_lang::solana_program::program_pack::Pack;
use anchor_spl::token_2022::spl_token_2022::state::Account as SplTokenAccount;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

#[derive(Accounts)]
pub struct DlmmClaimFees<'info> {
    #[account(
        seeds = [b"dlmm_vault".as_ref(), vault_account.owner.as_ref(), vault_account.dlmm_pool_id.as_ref()],
        bump
    )]
    pub vault_account: Account<'info, DlmmVaultAccount>,
    #[account(mut)]
    /// CHECK: The pool account
    pub lb_pair: UncheckedAccount<'info>,

    /// CHECK: Bin array extension account of the pool
    pub bin_array_bitmap_extension: Option<UncheckedAccount<'info>>,

    #[account(mut)]
    /// CHECK: Reserve account of token X
    pub reserve_x: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    /// CHECK: Reserve account of token Y
    pub reserve_y: InterfaceAccount<'info, TokenAccount>,

    #[account(mut)]
    /// CHECK: User token X account
    pub vault_token_x: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    /// CHECK: User token Y account
    pub vault_token_y: InterfaceAccount<'info, TokenAccount>,

    /// CHECK: Mint account of token X
    pub token_x_mint: InterfaceAccount<'info, Mint>,
    /// CHECK: Mint account of token Y
    pub token_y_mint: InterfaceAccount<'info, Mint>,

    #[account(mut)]
    /// CHECK: The position account to be created
    pub position: UncheckedAccount<'info>,

    // The owner or vault operator performing the add liquidity action
    pub signer: Signer<'info>,

    #[account(address = dlmm::ID)]
    /// CHECK: DLMM program
    pub dlmm_program: UncheckedAccount<'info>,

    /// CHECK: DLMM program event authority for event CPI
    pub event_authority: UncheckedAccount<'info>,

    /// CHECK: Token program of mint X
    pub token_x_program: Interface<'info, TokenInterface>,
    /// CHECK: Token program of mint Y
    pub token_y_program: Interface<'info, TokenInterface>,
    /// CHECK: Memo program
    pub memo_program: UncheckedAccount<'info>,
    /// CHECK: Bin array lower account
    #[account(mut)]
    pub bin_array_lower: UncheckedAccount<'info>,
    /// CHECK: Bin array upper account
    #[account(mut)]
    pub bin_array_upper: UncheckedAccount<'info>,
}

pub fn handle_dlmm_claim_fees<'a, 'b, 'c, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, DlmmClaimFees<'info>>,
    min_bin_id: i32,
    max_bin_id: i32,
) -> Result<()> {
    // We can only claim fees if we are in a position
    require!(
        ctx.accounts.vault_account.in_position,
        VaultErrorCode::PositionNotOpen
    );

    // Fee claiming is valid for both the owner and the operator
    let signer_is_owner = ctx.accounts.signer.key() == ctx.accounts.vault_account.owner;
    let signer_is_operator = ctx.accounts.signer.key() == ctx.accounts.vault_account.operator;
    if !signer_is_owner && !signer_is_operator {
        return Err(error!(VaultErrorCode::InvalidSigner));
    }

    let accounts = dlmm::cpi::accounts::ClaimFee2 {
        lb_pair: ctx.accounts.lb_pair.to_account_info(),
        reserve_x: ctx.accounts.reserve_x.to_account_info(),
        reserve_y: ctx.accounts.reserve_y.to_account_info(),
        user_token_x: ctx.accounts.vault_token_x.to_account_info(),
        user_token_y: ctx.accounts.vault_token_y.to_account_info(),
        token_x_mint: ctx.accounts.token_x_mint.to_account_info(),
        token_y_mint: ctx.accounts.token_y_mint.to_account_info(),
        position: ctx.accounts.position.to_account_info(),
        sender: ctx.accounts.vault_account.to_account_info(),
        token_program_x: ctx.accounts.token_x_program.to_account_info(),
        token_program_y: ctx.accounts.token_y_program.to_account_info(),
        event_authority: ctx.accounts.event_authority.to_account_info(),
        program: ctx.accounts.dlmm_program.to_account_info(),
        memo_program: ctx.accounts.memo_program.to_account_info(),
    };

    let signer_seeds: &[&[&[u8]]] = &[&[
        b"dlmm_vault",
        ctx.accounts.vault_account.owner.as_ref(),
        ctx.accounts.vault_account.dlmm_pool_id.as_ref(),
        &[ctx.bumps.vault_account],
    ]];

    let initial_x_balance = ctx.accounts.vault_token_x.amount;
    let initial_y_balance = ctx.accounts.vault_token_y.amount;

    let cpi_context = CpiContext::new(ctx.accounts.dlmm_program.to_account_info(), accounts)
        .with_signer(signer_seeds)
        .with_remaining_accounts(ctx.remaining_accounts.to_vec());

    // Explicitly have no support for any Token2022 hooks at this point in time. Vaults initialization ensures
    // that if the token is token2022, that it has no extensions
    let remaining_accounts_info = RemainingAccountsInfo { slices: vec![] };

    dlmm::cpi::claim_fee2(cpi_context, min_bin_id, max_bin_id, remaining_accounts_info)?;

    // Re-read the token accounts to determine the final balances
    let final_x_info = ctx.accounts.vault_token_x.to_account_info();
    let final_y_info = ctx.accounts.vault_token_y.to_account_info();
    let final_x_balance = token_amount(&final_x_info)?;
    let final_y_balance = token_amount(&final_y_info)?;

    emit!(ClaimFeesEvent {
        vault_account: ctx.accounts.vault_account.key(),
        position: ctx.accounts.position.key(),
        initial_x_balance: initial_x_balance,
        initial_y_balance: initial_y_balance,
        final_x_balance: final_x_balance,
        final_y_balance: final_y_balance,
        signer: ctx.accounts.signer.key(),
    });

    Ok(())
}

fn token_amount(ai: &anchor_lang::prelude::AccountInfo) -> Result<u64> {
    let amt = {
        let data = ai.try_borrow_data()?;
        SplTokenAccount::unpack(&data)?.amount
    };
    Ok(amt)
}
