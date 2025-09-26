use anyhow::Result;
use litesvm::LiteSVM;
use solana_client::client_error::ClientError;
use solana_message::AddressLookupTableAccount;
use solana_sdk::address_lookup_table::state::AddressLookupTable;
use solana_sdk::pubkey::Pubkey;

use crate::helpers::account::load_account;

pub fn get_address_lookup_table_accounts(
    svm: &mut LiteSVM,
    addresses: Vec<Pubkey>,
) -> Result<Vec<AddressLookupTableAccount>, ClientError> {
    let mut accounts = Vec::new();
    for key in addresses {
        let account = load_account(svm, &key);
        let address_lookup_table_account = AddressLookupTable::deserialize(&account.data).unwrap();
        accounts.push(AddressLookupTableAccount {
            key: key.to_bytes().into(),
            addresses: address_lookup_table_account
                .addresses
                .to_vec()
                .into_iter()
                .map(|a| a.to_bytes().into())
                .collect(),
        });
    }
    Ok(accounts)
}
