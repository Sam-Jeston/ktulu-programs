use anchor_lang::prelude::*;
use anchor_lang::solana_program::program_pack::Pack;
use anchor_spl::token_2022::spl_token_2022::state::Account as SplTokenAccount;

use crate::{DlmmVaultAccount, VaultErrorCode};

pub fn token_amount(ai: &anchor_lang::prelude::AccountInfo) -> Result<u64> {
    let amt = {
        let data = ai.try_borrow_data()?;
        SplTokenAccount::unpack(&data)?.amount
    };
    Ok(amt)
}

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

#[cfg(test)]
mod tests {
    use solana_sdk::{signature::Keypair, signer::Signer};

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
            lower_price_range_bps: 0,
            upper_price_range_bps: 0,
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
}
