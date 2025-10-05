use anchor_lang::prelude::*;

#[event]
#[derive(Debug)]
pub struct ClaimRewardsEvent {
    pub vault_account: Pubkey,
    pub position: Pubkey,
    pub reward_mint: Pubkey,
    pub initial_balance: u64,
    pub final_balance: u64,
    pub signer: Pubkey,
}
