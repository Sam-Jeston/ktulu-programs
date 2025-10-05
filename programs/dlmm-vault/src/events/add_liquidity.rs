use anchor_lang::prelude::*;

use crate::dlmm::types::BinLiquidityDistribution;

#[event]
#[derive(Debug)]
pub struct AddLiquidityEvent {
    pub vault_account: Pubkey,
    pub position: Pubkey,
    pub signer: Pubkey,
    pub token_x_amount: u64,
    pub token_y_amount: u64,
    pub bin_liquidity_dist: Vec<BinLiquidityDistribution>,
}
