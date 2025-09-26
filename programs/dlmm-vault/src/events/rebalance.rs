use anchor_lang::prelude::*;

#[event]
#[derive(Debug)]
pub struct RebalanceEvent {
    pub vault_account: Pubkey,
    pub in_mint: Pubkey,
    pub out_mint: Pubkey,
    pub initial_in_balance: u64,
    pub initial_out_balance: u64,
    pub final_in_balance: u64,
    pub final_out_balance: u64,
    pub signer: Pubkey,
}
