use anchor_lang::prelude::*;
#[error_code(offset = 0)]
pub enum VaultErrorCode {
    #[msg("Invalid token account")]
    InvalidTokenAccount = 401,
    #[msg("Invalid deposit amount")]
    InvalidDepositAmount = 402,
}
