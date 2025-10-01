use litesvm::LiteSVM;
use solana_account::Account;
use solana_sdk::{
    program_option::COption, program_pack::Pack, pubkey::Pubkey, signature::Keypair, signer::Signer,
};
use spl_associated_token_account::get_associated_token_address;
use spl_token::state::{Account as TokenAccount, AccountState};

pub fn create_and_fund_token_account(
    svm: &mut LiteSVM,
    user: &Keypair,
    mint: &Pubkey,
    amount: u64,
) -> Pubkey {
    let ata = get_associated_token_address(&user.pubkey(), &mint);

    let token_account = TokenAccount {
        mint: mint.clone(),
        owner: user.pubkey(),
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
            owner: anchor_spl::token::ID.to_bytes().into(),
            executable: false,
            rent_epoch: 0,
        },
    )
    .unwrap();

    ata
}
