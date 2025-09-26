use litesvm::LiteSVM;
use solana_account::Account;
use solana_pubkey::Pubkey as SPubkey;
use solana_sdk::{program_option::COption, program_pack::Pack, pubkey::Pubkey};
use spl_associated_token_account::get_associated_token_address_with_program_id;
use spl_token::state::{Account as TokenAccount, AccountState};

pub fn create_and_fund_token_account_by_pubkey(
    svm: &mut LiteSVM,
    user: &Pubkey,
    mint: &Pubkey,
    token_account_key: &Pubkey,
    amount: u64,
    token_program: &Pubkey,
) {
    let token_account = TokenAccount {
        mint: mint.clone(),
        owner: user.clone(),
        amount: amount,
        delegate: COption::None,
        state: AccountState::Initialized,
        is_native: COption::None,
        delegated_amount: 0,
        close_authority: COption::None,
    };

    let mut token_acc_bytes = [0u8; TokenAccount::LEN];
    TokenAccount::pack(token_account, &mut token_acc_bytes).unwrap();
    svm.set_account(
        token_account_key.to_bytes().into(),
        Account {
            lamports: 1_000_000_000,
            data: token_acc_bytes.to_vec(),
            owner: token_program.to_bytes().into(),
            executable: false,
            rent_epoch: 0,
        },
    )
    .unwrap();
}

pub fn create_and_fund_token_account(
    svm: &mut LiteSVM,
    user: &Pubkey,
    mint: &Pubkey,
    amount: u64,
    token_program: &Pubkey,
) -> Pubkey {
    let ata = get_associated_token_address_with_program_id(&user, &mint, token_program);

    let token_account = TokenAccount {
        mint: mint.clone(),
        owner: user.clone(),
        amount: amount,
        delegate: COption::None,
        state: AccountState::Initialized,
        is_native: COption::None,
        delegated_amount: 0,
        close_authority: COption::None,
    };

    let mut token_acc_bytes = [0u8; TokenAccount::LEN];
    TokenAccount::pack(token_account, &mut token_acc_bytes).unwrap();
    svm.set_account(
        ata.to_bytes().into(),
        Account {
            lamports: 1_000_000_000,
            data: token_acc_bytes.to_vec(),
            owner: token_program.to_bytes().into(),
            executable: false,
            rent_epoch: 0,
        },
    )
    .unwrap();

    ata
}

pub fn validate_token_account_balance(svm: &mut LiteSVM, ata: &Pubkey, amount: u64) {
    let pubkey = SPubkey::from(ata.to_bytes());
    let token_account = svm.get_account(&pubkey).unwrap();
    let token_account_data = TokenAccount::unpack_from_slice(&token_account.data).unwrap();
    assert_eq!(token_account_data.amount, amount);
}
