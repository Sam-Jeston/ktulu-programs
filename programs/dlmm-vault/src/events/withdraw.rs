use anchor_lang::prelude::*;

#[event]
#[derive(Debug)]
pub struct WithdrawEvent {
    pub vault_account: Pubkey,
    pub token_x_withdraw_amount: u64,
    pub token_y_withdraw_amount: u64,
}
