use anchor_lang::prelude::*;

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone, Debug, Eq, PartialEq)]
pub enum FeeCompoundingStrategy {
    Aggressive,
    Conservative,
}

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone, Debug, Eq, PartialEq)]
pub enum VolatilityStrategy {
    Spot,
    Curve,
    BidAsk,
}

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
    pub auto_compound: bool,
    pub auto_rebalance: bool,
    // Swapping harvested token X and token Y into harvest mint is optional
    pub use_harvest_mint: bool,
    // How many basis points of the fees claimed should be harvested on fee claim
    pub harvest_bps: u16,
    // The pubkey of the mint to harvest into. It can be either Token X, Token Y or a different mint
    pub harvest_mint: Pubkey,
    pub fee_compounding_strategy: FeeCompoundingStrategy,
    pub bin_width: u16,
    pub volatility_strategy: VolatilityStrategy,
    pub virtual_token_x_harvest: u64,
    pub virtual_token_y_harvest: u64,
}

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone, Debug, Eq, PartialEq)]
pub enum SingleSidedStrategy {
    // no rebalancing. Can have auto-compounding and auto-harvesting; not mutually exclusive
    BidOnly,
    // no rebalancing. Can have auto-compounding and auto-harvesting; not mutually exclusive
    AskOnly,
    // Exit balance to vault, as 100% token Y if active bin falls below min bin
    BidWithStoplossOnDownside,
}
