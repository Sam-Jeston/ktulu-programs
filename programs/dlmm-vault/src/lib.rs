// https://github.com/coral-xyz/anchor/issues/3401#issuecomment-2513466441
#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

pub mod instructions;
pub use instructions::*;

pub mod program_accounts;
pub use program_accounts::*;

pub mod errors;
pub use errors::*;

pub mod events;
pub use events::*;

declare_program!(dlmm);
use crate::dlmm::types::BinLiquidityDistribution;

declare_id!("7Y1iiXP68seqhZtyQ1fEwxCYJVmJztwvXBBnZvRn3DyC");

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

    pub fn dlmm_withdraw<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, DlmmWithdraw<'info>>,
        token_x_withdraw_amount: u64,
        token_y_withdraw_amount: u64,
    ) -> Result<()> {
        instructions::withdraw::handle_dlmm_withdraw(
            ctx,
            token_x_withdraw_amount,
            token_y_withdraw_amount,
        )
    }

    pub fn create_position<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, DlmmCreatePosition<'info>>,
        lower_bin_id: i32,
        width: i32,
    ) -> Result<()> {
        instructions::create_position::handle_dlmm_create_position(ctx, lower_bin_id, width)
    }

    pub fn add_liquidity<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, DlmmAddLiquidity<'info>>,
        amount_x: u64,
        amount_y: u64,
        bin_liquidity_dist: Vec<BinLiquidityDistribution>,
    ) -> Result<()> {
        instructions::add_liquidity::handle_dlmm_add_liquidity(
            ctx,
            amount_x,
            amount_y,
            bin_liquidity_dist,
        )
    }
}
