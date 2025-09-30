use solana_program_test::ProgramTest;
use solana_sdk::{
    account::{Account, AccountSharedData, WritableAccount},
    pubkey::Pubkey,
};
use std::{fs, str::FromStr};

pub fn load_fixture(test: &mut ProgramTest, key: &str) -> Account {
    let data: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(format!("tests/fixtures/{key}.json")).unwrap())
            .unwrap();
    let lamports = data["lamports"].as_u64().unwrap();
    let owner = Pubkey::from_str(data["owner"].as_str().unwrap()).unwrap();
    let account_data = base64::decode(data["data"][0].as_str().unwrap()).unwrap();
    let rent_epoch = data["rent_epoch"].as_u64().unwrap();
    let account = AccountSharedData::create(
        lamports,
        account_data,
        owner,
        data["executable"].as_bool().unwrap(),
        rent_epoch,
    );

    account.into()
}
