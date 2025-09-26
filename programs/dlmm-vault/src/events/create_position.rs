use anchor_lang::prelude::*;

#[event]
#[derive(Debug)]
pub struct CreatePositionEvent {
    pub vault_account: Pubkey,
    pub position: Pubkey,
    pub signer: Pubkey,
    pub lower_bin_id: i32,
    pub width: i32,
}
