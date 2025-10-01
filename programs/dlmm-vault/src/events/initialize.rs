use anchor_lang::prelude::*;

#[event]
#[derive(Debug)]
pub struct InitializeEvent {
    pub vault_account: Pubkey,
    pub owner: Pubkey,
    pub token_x_mint: Pubkey,
    pub token_y_mint: Pubkey,
    pub dlmm_pool: Pubkey,
    pub lower_price_range_bps: u64,
    pub upper_price_range_bps: u64,
    pub operator: Pubkey,
    pub position_id: Pubkey,
}
