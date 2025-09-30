use std::{fs, str::FromStr};

use solana_account::{Account, WritableAccount};
use solana_pubkey::Pubkey;

pub fn load_account_fixture(key: &str) -> Account {
    let data: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(format!("tests/fixtures/{key}.json")).unwrap())
            .unwrap();
    let lamports = data["account"]["lamports"].as_u64().unwrap();
    let owner = Pubkey::from_str(data["account"]["owner"].as_str().unwrap()).unwrap();
    let account_data = base64::decode(data["account"]["data"][0].as_str().unwrap()).unwrap();
    let rent_epoch = data["account"]["rentEpoch"].as_u64().unwrap();
    WritableAccount::create(
        lamports,
        account_data,
        owner,
        data["account"]["executable"].as_bool().unwrap(),
        rent_epoch,
    )
}
