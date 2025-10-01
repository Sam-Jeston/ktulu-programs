use anchor_lang::prelude::*;

#[event]
#[derive(Debug)]
pub struct DepositEvent {
    pub vault_account: Pubkey,
    pub token_x_deposit_amount: u64,
    pub token_y_deposit_amount: u64,
}
