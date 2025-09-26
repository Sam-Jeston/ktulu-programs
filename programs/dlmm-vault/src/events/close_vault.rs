use anchor_lang::prelude::*;

#[event]
#[derive(Debug)]
pub struct CloseVaultEvent {
    pub vault_account: Pubkey,
}
