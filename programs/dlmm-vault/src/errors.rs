use anchor_lang::prelude::*;
#[error_code]
pub enum VaultErrorCode {
    #[msg("Invalid token account")]
    InvalidTokenAccount,
    #[msg("Invalid deposit amount")]
    InvalidDepositAmount,
    #[msg("Invalid signer")]
    InvalidSigner,
    #[msg("Trying to withdraw more than the vault balance for token")]
    InvalidWithdrawAmount,
    #[msg("Position still open")]
    PositionStillOpen,
    #[msg("Position not open")]
    PositionNotOpen,
    #[msg("Non-zero balance")]
    NonZeroBalance,
}
