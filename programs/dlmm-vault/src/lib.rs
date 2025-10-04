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
declare_program!(jupiter);
use crate::dlmm::types::{BinLiquidityDistribution, BinLiquidityReduction};

declare_id!("7Y1iiXP68seqhZtyQ1fEwxCYJVmJztwvXBBnZvRn3DyC");

#[program]
pub mod dlmm_vault {

    use crate::dlmm::types::BinLiquidityReduction;

    use super::*;

    pub fn initialize<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, Initialize<'info>>,
        lower_price_range_bps: u64,
        upper_price_range_bps: u64,
        operator: Pubkey,
    ) -> Result<()> {
        instructions::initialize::handle_initialize(
            ctx,
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

    // TODO: Integration tests
    pub fn remove_liquidity<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, DlmmRemoveLiquidity<'info>>,
        bin_liquidity_reduction: Vec<BinLiquidityReduction>,
    ) -> Result<()> {
        instructions::remove_liquidity::handle_dlmm_remove_liquidity(ctx, bin_liquidity_reduction)
    }

    // TODO: Integration tests
    pub fn claim_fees<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, DlmmClaimFees<'info>>,
        min_bin_id: i32,
        max_bin_id: i32,
    ) -> Result<()> {
        instructions::claim_fees::handle_dlmm_claim_fees(ctx, min_bin_id, max_bin_id)
    }

    pub fn rebalance<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, Rebalance<'info>>,
        jup_swap_data: Vec<u8>,
    ) -> Result<()> {
        instructions::rebalance::handle_rebalance(ctx, jup_swap_data)
    }

    // TODO: Integration tests
    pub fn close_vault<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, CloseVault<'info>>,
    ) -> Result<()> {
        instructions::close::handle_close_vault(ctx)
    }
}
