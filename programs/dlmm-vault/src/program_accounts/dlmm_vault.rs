use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct DlmmVaultAccount {
    // This owner will lets us effectively memcmp filter for vault accounts per user
    pub owner: Pubkey,
    pub in_position: bool,
    pub operator: Pubkey,
    pub dlmm_pool_id: Pubkey,
    pub position_id: Pubkey,
    pub token_x_mint: Pubkey,
    pub token_y_mint: Pubkey,
    pub lower_price_range_bps: u64,
    pub upper_price_range_bps: u64,
}
