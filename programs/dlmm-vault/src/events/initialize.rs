use anchor_lang::prelude::*;

use crate::{FeeCompoundingStrategy, VolatilityStrategy};

#[event]
#[derive(Debug)]
pub struct InitializeEvent {
    pub vault_account: Pubkey,
    pub owner: Pubkey,
    pub token_x_mint: Pubkey,
    pub token_y_mint: Pubkey,
    pub dlmm_pool: Pubkey,
    pub auto_compound: bool,
    pub auto_rebalance: bool,
    pub fee_compounding_strategy: FeeCompoundingStrategy,
    pub volatility_strategy: VolatilityStrategy,
    pub bin_width: u16,
    pub operator: Pubkey,
    pub position_id: Pubkey,
    pub use_harvest_mint: bool,
    pub harvest_bps: u16,
    pub harvest_mint: Pubkey,
    pub amount_x: u64,
    pub amount_y: u64,
}
