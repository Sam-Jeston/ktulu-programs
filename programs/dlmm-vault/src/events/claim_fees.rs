use anchor_lang::prelude::*;

#[event]
#[derive(Debug)]
pub struct ClaimFeesEvent {
    pub vault_account: Pubkey,
    pub position: Pubkey,
    pub y_compounded: u64,
    pub x_compounded: u64,
    pub y_fee_paid: u64,
    pub x_harvested: u64,
    pub y_harvested: u64,
    pub signer: Pubkey,
}
