use crate::{
    dlmm::{
        self,
        types::{BinLiquidityDistribution, LiquidityParameter},
    },
    events::add_liquidity::AddLiquidityEvent,
    DlmmVaultAccount, VaultErrorCode,
};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct DlmmAddLiquidity<'info> {
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
    pub reserve_x: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: Reserve account of token Y
    pub reserve_y: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: User token X account
    pub vault_token_x: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: User token Y account
    pub vault_token_y: UncheckedAccount<'info>,

    /// CHECK: Mint account of token X
    pub token_x_mint: UncheckedAccount<'info>,
    /// CHECK: Mint account of token Y
    pub token_y_mint: UncheckedAccount<'info>,

    /// CHECK: Oracle account of the pool
    pub oracle: UncheckedAccount<'info>,

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
    pub token_x_program: UncheckedAccount<'info>,
    /// CHECK: Token program of mint Y
    pub token_y_program: UncheckedAccount<'info>,
    /// CHECK: Bin array lower account
    #[account(mut)]
    pub bin_array_lower: UncheckedAccount<'info>,
    /// CHECK: Bin array upper account
    #[account(mut)]
    pub bin_array_upper: UncheckedAccount<'info>,
}

pub fn handle_dlmm_add_liquidity<'a, 'b, 'c, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, DlmmAddLiquidity<'info>>,
    amount_x: u64,
    amount_y: u64,
    bin_liquidity_dist: Vec<BinLiquidityDistribution>,
) -> Result<()> {
    // Position creation is valid for both the owner and the operator
    let signer_is_owner = ctx.accounts.signer.key() == ctx.accounts.vault_account.owner;
    let signer_is_operator = ctx.accounts.signer.key() == ctx.accounts.vault_account.operator;
    if !signer_is_owner && !signer_is_operator {
        return Err(error!(VaultErrorCode::InvalidSigner));
    }

    let accounts = dlmm::cpi::accounts::AddLiquidity {
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
        bin_array_lower: ctx.accounts.bin_array_lower.to_account_info(),
        bin_array_upper: ctx.accounts.bin_array_upper.to_account_info(),
    };

    let signer_seeds: &[&[&[u8]]] = &[&[
        b"dlmm_vault",
        ctx.accounts.vault_account.owner.as_ref(),
        ctx.accounts.vault_account.dlmm_pool_id.as_ref(),
        &[ctx.bumps.vault_account],
    ]];

    let cpi_context = CpiContext::new(ctx.accounts.dlmm_program.to_account_info(), accounts)
        .with_signer(signer_seeds)
        .with_remaining_accounts(ctx.remaining_accounts.to_vec());

    let liquidity_parameter = LiquidityParameter {
        amount_x,
        amount_y,
        bin_liquidity_dist: bin_liquidity_dist.clone(),
    };

    dlmm::cpi::add_liquidity(cpi_context, liquidity_parameter)?;

    emit!(AddLiquidityEvent {
        vault_account: ctx.accounts.vault_account.key(),
        token_x_amount: amount_x,
        token_y_amount: amount_y,
        bin_liquidity_dist,
    });

    Ok(())
}
