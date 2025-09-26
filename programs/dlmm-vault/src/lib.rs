// https://github.com/coral-xyz/anchor/issues/3401#issuecomment-2513466441
#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;

pub mod instructions;
pub use instructions::*;

declare_program!(dlmm);

use crate::dlmm_swap::*;

declare_id!("4JTNRRQpgLusbEhGnzTuE9kgPgMLXQX1wqBzU52GduqH");

#[program]
pub mod dlmm_vault {
    use super::*;

    pub fn dlmm_swap<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, DlmmSwap<'info>>,
        amount_in: u64,
        min_amount_out: u64,
    ) -> Result<()> {
        instructions::dlmm_cpi::dlmm_swap::handle_dlmm_swap(ctx, amount_in, min_amount_out)
    }
}
