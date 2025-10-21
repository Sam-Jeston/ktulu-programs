use anchor_lang::prelude::*;
use anchor_lang::solana_program::program_pack::Pack;

use crate::{DlmmVaultAccount, VaultErrorCode};

pub fn ensure_signer_is_owner(signer: &Pubkey, vault_account: &DlmmVaultAccount) -> Result<()> {
    let signer_is_owner = signer.key() == vault_account.owner;
    if !signer_is_owner {
        return Err(error!(VaultErrorCode::InvalidSigner));
    }

    Ok(())
}

pub fn ensure_signer_is_owner_or_operator(
    signer: &Pubkey,
    vault_account: &DlmmVaultAccount,
) -> Result<()> {
    let signer_is_owner = signer.key() == vault_account.owner;
    let signer_is_operator = signer.key() == vault_account.operator;
    if !signer_is_owner && !signer_is_operator {
        return Err(error!(VaultErrorCode::InvalidSigner));
    }

    Ok(())
}

#[inline]
pub fn mul_div_floor_u64(val: u64, numerator: u64, denom: u64) -> u64 {
    // Use u128 intermediates to avoid precision loss and overflow, then cast back down.
    ((val as u128) * (numerator as u128) / (denom as u128)) as u64
}

#[cfg(test)]
mod tests {
    use solana_sdk::{signature::Keypair, signer::Signer};

    use crate::{FeeCompoundingStrategy, VolatilityStrategy};

    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    fn seed_vault_account(owner: Pubkey, operator: Pubkey) -> DlmmVaultAccount {
        DlmmVaultAccount {
            owner,
            operator,
            dlmm_pool_id: Pubkey::default(),
            in_position: false,
            position_id: Pubkey::default(),
            token_x_mint: Pubkey::default(),
            token_y_mint: Pubkey::default(),
            auto_compound: true,
            auto_rebalance: true,
            volatility_strategy: VolatilityStrategy::Spot,
            bin_width: 40,
            fee_compounding_strategy: FeeCompoundingStrategy::Aggressive,
            use_harvest_mint: false,
            harvest_bps: 0,
            harvest_mint: Pubkey::default(),
            virtual_token_x_harvest: 0,
            virtual_token_y_harvest: 0,
        }
    }

    #[test]
    fn test_ensure_signer_is_owner() {
        let owner = Keypair::new();
        let vault_account = seed_vault_account(owner.pubkey(), owner.pubkey());
        assert_eq!(
            ensure_signer_is_owner(&owner.pubkey(), &vault_account),
            Ok(())
        );
    }

    #[test]
    fn test_ensure_signer_is_owner_throws_when_not_owner() {
        let owner = Keypair::new();
        let random_signer = Keypair::new();
        let vault_account = seed_vault_account(owner.pubkey(), owner.pubkey());
        assert_eq!(
            ensure_signer_is_owner(&random_signer.pubkey(), &vault_account),
            Err(error!(VaultErrorCode::InvalidSigner))
        );
    }

    #[test]
    fn test_ensure_signer_is_owner_or_operator_as_owner() {
        let owner = Keypair::new();
        let operator = Keypair::new();
        let vault_account = seed_vault_account(owner.pubkey(), operator.pubkey());
        assert_eq!(
            ensure_signer_is_owner_or_operator(&owner.pubkey(), &vault_account),
            Ok(())
        );
    }

    #[test]
    fn test_ensure_signer_is_owner_or_operator_as_operator() {
        let owner = Keypair::new();
        let operator = Keypair::new();
        let vault_account = seed_vault_account(owner.pubkey(), operator.pubkey());
        assert_eq!(
            ensure_signer_is_owner_or_operator(&operator.pubkey(), &vault_account),
            Ok(())
        );
    }

    #[test]
    fn test_ensure_signer_is_owner_or_operator_throws_when_not_owner_or_operator() {
        let owner = Keypair::new();
        let operator = Keypair::new();
        let random_signer = Keypair::new();
        let vault_account = seed_vault_account(owner.pubkey(), operator.pubkey());
        assert_eq!(
            ensure_signer_is_owner_or_operator(&random_signer.pubkey(), &vault_account),
            Err(error!(VaultErrorCode::InvalidSigner))
        );
    }

    #[test]
    fn test_mul_div_floor_u64() {
        assert_eq!(mul_div_floor_u64(1000, 50, 10_000), 5);
        assert_eq!(mul_div_floor_u64(12000, 50, 10_000), 60);
    }

    #[test]
    fn test_mul_div_floor_u64_does_not_overflow() {
        assert_eq!(
            mul_div_floor_u64(u64::MAX, 5, 10_000),
            u64::MAX / 10_000 * 5
        );
    }
}
