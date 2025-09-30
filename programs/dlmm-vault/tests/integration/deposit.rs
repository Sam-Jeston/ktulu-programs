use anchor_lang::{InstructionData, ToAccountMetas};
use litesvm::LiteSVM;
use solana_account::Account;
use solana_instruction::account_meta::AccountMeta as SAccountMeta;
use solana_keypair::Keypair as SKeypair;
use solana_message::Instruction;
use solana_message::{Message, VersionedMessage};
use solana_pubkey::pubkey as spubkey;
use solana_sdk::program_option::COption;
use solana_sdk::program_pack::Pack;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::{signature::Keypair, signer::Signer};
use solana_transaction::versioned::VersionedTransaction;
use spl_associated_token_account::get_associated_token_address;
use spl_token::state::{Account as TokenAccount, AccountState};

const USDC_USDT_POOL: Pubkey = solana_sdk::pubkey!("ARwi1S4DaiTG5DX7S4M4ZsrXqpMD1MrTmbu9ue2tpmEq");
const USDC_MINT: Pubkey = solana_sdk::pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const USDT_MINT: Pubkey = solana_sdk::pubkey!("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB");

#[test]
fn test_deposit() {
    let mock_user2 = SKeypair::new();
    let mock_user = Keypair::from_bytes(&mock_user2.to_bytes()).unwrap();

    let ata_x = get_associated_token_address(&mock_user.pubkey(), &USDC_MINT);
    let ata_y = get_associated_token_address(&mock_user.pubkey(), &USDT_MINT);

    let mut svm = LiteSVM::new();
    let program_bytes = include_bytes!("../../../../target/deploy/dlmm_vault.so");
    let program_id = spubkey!("7Y1iiXP68seqhZtyQ1fEwxCYJVmJztwvXBBnZvRn3DyC");
    svm.add_program(program_id, program_bytes).unwrap();
    svm.airdrop(&mock_user.pubkey().to_bytes().into(), 1_000_000_000)
        .unwrap();

    let token_x_account = TokenAccount {
        mint: USDC_MINT,
        owner: mock_user.pubkey(),
        amount: 1_000_000_000_000,
        delegate: COption::None,
        state: AccountState::Initialized,
        is_native: COption::None,
        delegated_amount: 0,
        close_authority: COption::None,
    };

    let mut token_acc_bytes = [0u8; TokenAccount::LEN];
    TokenAccount::pack(token_x_account, &mut token_acc_bytes).unwrap();
    svm.set_account(
        ata_x.to_bytes().into(),
        Account {
            lamports: 1_000_000_000,
            data: token_acc_bytes.to_vec(),
            owner: anchor_spl::token::ID.to_bytes().into(),
            executable: false,
            rent_epoch: 0,
        },
    )
    .unwrap();

    let ix_data = dlmm_vault::instruction::DlmmDeposit {
        token_x_deposit_amount: 1_000_000,
        token_y_deposit_amount: 0,
    }
    .data();

    let accounts = dlmm_vault::accounts::DlmmDeposit {
        vault_account: USDC_USDT_POOL,
        signer: mock_user.pubkey(),
        vault_owner_token_x: ata_x,
        vault_token_x_account: ata_x,
        vault_owner_token_y: ata_y,
        vault_token_y_account: ata_y,
        token_x_mint: USDC_MINT,
        token_y_mint: USDT_MINT,
        token_x_program: anchor_spl::token::ID,
        token_y_program: anchor_spl::token::ID,
    }
    .to_account_metas(None);

    let instruction = Instruction {
        program_id: dlmm_vault::id().to_bytes().into(),
        data: ix_data,
        accounts: accounts
            .iter()
            .map(|a| SAccountMeta {
                pubkey: a.pubkey.to_bytes().into(),
                is_signer: a.is_signer,
                is_writable: a.is_writable,
            })
            .collect(),
    };

    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(
        &[instruction],
        Some(&mock_user.pubkey().to_bytes().into()),
        &blockhash,
    );
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&mock_user2]).unwrap();
    // let's sim it first
    let sim_res = svm.simulate_transaction(tx.clone()).unwrap();
    let meta = svm.send_transaction(tx).unwrap();
    assert_eq!(sim_res.meta, meta);
    assert_eq!(meta.logs[1], "Program log: static string");
    assert!(meta.compute_units_consumed < 10_000)
}
