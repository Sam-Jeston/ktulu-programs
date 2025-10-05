use crate::{
    dlmm::{
        self,
        types::{BinLiquidityReduction, RemainingAccountsInfo},
    },
    ensure_signer_is_owner_or_operator,
    events::remove_liquidity::RemoveLiquidityEvent,
    DlmmVaultAccount, VaultErrorCode,
};
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

#[derive(Accounts)]
pub struct DlmmRemoveLiquidity<'info> {
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

pub fn handle_dlmm_remove_liquidity<'a, 'b, 'c, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, DlmmRemoveLiquidity<'info>>,
    bin_liquidity_reduction: Vec<BinLiquidityReduction>,
) -> Result<()> {
    // Position creation is valid for both the owner and the operator
    ensure_signer_is_owner_or_operator(&ctx.accounts.signer.key, &ctx.accounts.vault_account)?;

    let accounts = dlmm::cpi::accounts::RemoveLiquidity2 {
        lb_pair: ctx.accounts.lb_pair.to_account_info(),
        bin_array_bitmap_extension: ctx
            .accounts
            .bin_array_bitmap_extension
            .as_ref()
            .map(|account| account.to_account_info()),
        reserve_x: ctx.accounts.reserve_x.to_account_info(),
        reserve_y: ctx.accounts.reserve_y.to_account_info(),
        user_token_x: ctx.accounts.vault_token_x.to_account_info(),
        user_token_y: ctx.accounts.vault_token_y.to_account_info(),
        token_x_mint: ctx.accounts.token_x_mint.to_account_info(),
        token_y_mint: ctx.accounts.token_y_mint.to_account_info(),
        position: ctx.accounts.position.to_account_info(),
        sender: ctx.accounts.vault_account.to_account_info(),
        token_x_program: ctx.accounts.token_x_program.to_account_info(),
        token_y_program: ctx.accounts.token_y_program.to_account_info(),
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

    let cpi_context = CpiContext::new(ctx.accounts.dlmm_program.to_account_info(), accounts)
        .with_signer(signer_seeds);

    // Explicitly have no support for any Token2022 hooks at this point in time. Vaults initialization ensures
    // that if the token is token2022, that it has no extensions
    let remaining_accounts_info = RemainingAccountsInfo { slices: vec![] };

    dlmm::cpi::remove_liquidity2(
        cpi_context,
        bin_liquidity_reduction.clone(),
        remaining_accounts_info,
    )?;

    emit!(RemoveLiquidityEvent {
        vault_account: ctx.accounts.vault_account.key(),
        signer: ctx.accounts.signer.key(),
        bin_liquidity_reduction,
    });

    Ok(())
}
