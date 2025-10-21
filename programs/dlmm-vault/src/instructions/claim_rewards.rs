use crate::{
    dlmm::{self, types::RemainingAccountsInfo},
    ensure_signer_is_owner_or_operator,
    events::claim_rewards::ClaimRewardsEvent,
    DlmmVaultAccount, VaultErrorCode,
};
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

#[derive(Accounts)]
pub struct DlmmClaimRewards<'info> {
    #[account(
        seeds = [b"dlmm_vault".as_ref(), vault_account.owner.as_ref(), vault_account.dlmm_pool_id.as_ref()],
        bump
    )]
    pub vault_account: Account<'info, DlmmVaultAccount>,
    #[account(mut)]
    /// CHECK: The pool account
    pub lb_pair: UncheckedAccount<'info>,

    #[account(mut)]
    pub reward_vault: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = reward_mint,
        associated_token::authority = vault_account,
        associated_token::token_program = token_program
    )]
    pub vault_token_account: InterfaceAccount<'info, TokenAccount>,

    pub reward_mint: InterfaceAccount<'info, Mint>,

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
    pub token_program: Interface<'info, TokenInterface>,
    /// CHECK: Memo program
    pub memo_program: UncheckedAccount<'info>,
}

pub fn handle_dlmm_claim_rewards<'a, 'b, 'c, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, DlmmClaimRewards<'info>>,
    reward_index: u64,
    min_bin_id: i32,
    max_bin_id: i32,
) -> Result<()> {
    // We can only claim fees if we are in a position
    require!(
        ctx.accounts.vault_account.in_position,
        VaultErrorCode::PositionNotOpen
    );

    // Fee claiming is valid for both the owner and the operator
    ensure_signer_is_owner_or_operator(&ctx.accounts.signer.key, &ctx.accounts.vault_account)?;

    let accounts = dlmm::cpi::accounts::ClaimReward2 {
        lb_pair: ctx.accounts.lb_pair.to_account_info(),
        reward_vault: ctx.accounts.reward_vault.to_account_info(),
        user_token_account: ctx.accounts.vault_token_account.to_account_info(),
        reward_mint: ctx.accounts.reward_mint.to_account_info(),
        position: ctx.accounts.position.to_account_info(),
        sender: ctx.accounts.vault_account.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
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

    let initial_balance = ctx.accounts.vault_token_account.amount;

    let cpi_context = CpiContext::new(ctx.accounts.dlmm_program.to_account_info(), accounts)
        .with_signer(signer_seeds);

    // Explicitly have no support for any Token2022 hooks at this point in time
    let remaining_accounts_info = RemainingAccountsInfo { slices: vec![] };

    dlmm::cpi::claim_reward2(
        cpi_context,
        reward_index,
        min_bin_id,
        max_bin_id,
        remaining_accounts_info,
    )?;

    // Re-read the token accounts to determine the final balances
    ctx.accounts.vault_token_account.reload()?;
    let final_balance = ctx.accounts.vault_token_account.amount;

    emit!(ClaimRewardsEvent {
        vault_account: ctx.accounts.vault_account.key(),
        position: ctx.accounts.position.key(),
        reward_mint: ctx.accounts.reward_mint.key(),
        initial_balance: initial_balance,
        final_balance: final_balance,
        signer: ctx.accounts.signer.key(),
    });

    Ok(())
}
