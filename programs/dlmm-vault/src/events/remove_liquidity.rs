use anchor_lang::prelude::*;

use crate::dlmm::types::BinLiquidityReduction;

#[event]
#[derive(Debug)]
pub struct RemoveLiquidityEvent {
    pub vault_account: Pubkey,
    pub bin_liquidity_reduction: Vec<BinLiquidityReduction>,
}
