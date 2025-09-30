// https://github.com/coral-xyz/anchor/issues/3401#issuecomment-2513466441
#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;

pub mod instructions;
use anchor_spl::token::{Mint, Token, TokenAccount};
pub use instructions::*;

pub mod program_accounts;
pub use program_accounts::*;

pub mod errors;
pub use errors::*;

declare_program!(dlmm);

declare_id!("4JTNRRQpgLusbEhGnzTuE9kgPgMLXQX1wqBzU52GduqH");

#[program]
pub mod dlmm_vault {
    use super::*;

    pub fn initialize<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, Initialize<'info>>,
        token_x_mint: Pubkey,
        token_y_mint: Pubkey,
        lower_price_range_bps: u64,
        upper_price_range_bps: u64,
        operator: Pubkey,
    ) -> Result<()> {
        instructions::initialize::handle_initialize(
            ctx,
            token_x_mint,
            token_y_mint,
            lower_price_range_bps,
            upper_price_range_bps,
            operator,
        )
    }

    pub fn dlmm_deposit<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, DlmmDeposit<'info>>,
        token_x_deposit_amount: u64,
        token_y_deposit_amount: u64,
    ) -> Result<()> {
        instructions::deposit::handle_dlmm_deposit(
            ctx,
            token_x_deposit_amount,
            token_y_deposit_amount,
        )
    }

    pub fn dlmm_swap<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, DlmmSwap<'info>>,
        amount_in: u64,
        min_amount_out: u64,
    ) -> Result<()> {
        instructions::swap::handle_dlmm_swap(ctx, amount_in, min_amount_out)
    }
}
