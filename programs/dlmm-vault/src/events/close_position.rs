use anchor_lang::prelude::*;

#[event]
#[derive(Debug)]
pub struct ClosePositionEvent {
    pub vault_account: Pubkey,
    pub position: Pubkey,
    pub signer: Pubkey,
}
