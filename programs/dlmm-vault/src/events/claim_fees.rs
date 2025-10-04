use anchor_lang::prelude::*;

#[event]
#[derive(Debug)]
pub struct ClaimFeesEvent {
    pub vault_account: Pubkey,
    pub position: Pubkey,
    pub initial_x_balance: u64,
    pub initial_y_balance: u64,
    pub final_x_balance: u64,
    pub final_y_balance: u64,
    pub signer: Pubkey,
}
