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
    #[msg("Position width does not match vault configuration")]
    InvalidWidth,
    #[msg("Invalid harvest bps - maximum is 10_000")]
    InvalidHarvestBps,
    #[msg("Auto-rebalance or harvest is not enabled - swapping not supported")]
    AutoRebalanceOrHarvestNotEnabled,
}
