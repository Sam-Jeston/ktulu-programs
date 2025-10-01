use anchor_lang::solana_program::hash::hashv;
use anchor_lang::{system_program, AnchorDeserialize, InstructionData, ToAccountMetas};
use base64::decode;
use dlmm_vault::events::initialize::InitializeEvent;
use litesvm::LiteSVM;
use solana_account::Account;
use solana_instruction::account_meta::AccountMeta as SAccountMeta;
use solana_keypair::{Keypair as SKeypair, Signer as SSigner};
use solana_message::Instruction;
use solana_message::{Message, VersionedMessage};
use solana_pubkey::pubkey as spubkey;
use solana_sdk::program_option::COption;
use solana_sdk::program_pack::Pack;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::{signature::Keypair, signer::Signer};
use solana_transaction::versioned::VersionedTransaction;
use spl_associated_token_account::{
    get_associated_token_address, get_associated_token_address_with_program_id,
};
use spl_token::state::{Account as TokenAccount, AccountState};

use crate::helpers::load_fixture::load_account_fixture;

const USDC_USDT_POOL: Pubkey = solana_sdk::pubkey!("ARwi1S4DaiTG5DX7S4M4ZsrXqpMD1MrTmbu9ue2tpmEq");
const USDC_MINT: Pubkey = solana_sdk::pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const USDT_MINT: Pubkey = solana_sdk::pubkey!("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB");

#[test]
fn test_initialize() {
    let user = SKeypair::new();
    let mock_user = Keypair::from_bytes(&user.to_bytes()).unwrap();

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

    let mut token_x_acc_bytes = [0u8; TokenAccount::LEN];
    TokenAccount::pack(token_x_account, &mut token_x_acc_bytes).unwrap();
    svm.set_account(
        ata_x.to_bytes().into(),
        Account {
            lamports: 1_000_000_000,
            data: token_x_acc_bytes.to_vec(),
            owner: anchor_spl::token::ID.to_bytes().into(),
            executable: false,
            rent_epoch: 0,
        },
    )
    .unwrap();

    let token_y_account = TokenAccount {
        mint: USDT_MINT,
        owner: mock_user.pubkey(),
        amount: 1_000_000_000_000,
        delegate: COption::None,
        state: AccountState::Initialized,
        is_native: COption::None,
        delegated_amount: 0,
        close_authority: COption::None,
    };

    let mut token_y_acc_bytes = [0u8; TokenAccount::LEN];
    TokenAccount::pack(token_y_account, &mut token_y_acc_bytes).unwrap();
    svm.set_account(
        ata_y.to_bytes().into(),
        Account {
            lamports: 1_000_000_000,
            data: token_y_acc_bytes.to_vec(),
            owner: anchor_spl::token::ID.to_bytes().into(),
            executable: false,
            rent_epoch: 0,
        },
    )
    .unwrap();

    let dlmm_pool = load_account_fixture(USDC_USDT_POOL.to_string().as_str());
    svm.set_account(USDC_USDT_POOL.to_bytes().into(), dlmm_pool)
        .unwrap();

    let token_x_mint = load_account_fixture(USDC_MINT.to_string().as_str());
    svm.set_account(USDC_MINT.to_bytes().into(), token_x_mint)
        .unwrap();

    let token_y_mint = load_account_fixture(USDT_MINT.to_string().as_str());
    svm.set_account(USDT_MINT.to_bytes().into(), token_y_mint)
        .unwrap();

    let ix_data = dlmm_vault::instruction::Initialize {
        token_x_mint: USDC_MINT,
        token_y_mint: USDT_MINT,
        lower_price_range_bps: 0,
        upper_price_range_bps: 0,
        operator: mock_user.pubkey(),
    }
    .data();

    let (vault_pda, _bump) = Pubkey::find_program_address(
        &[
            b"dlmm_vault",
            mock_user.pubkey().as_ref(),
            USDC_USDT_POOL.as_ref(), // dlmm_pool.key
        ],
        &dlmm_vault::id(),
    );

    let vault_ata_x = get_associated_token_address_with_program_id(
        &vault_pda,
        &USDC_MINT,
        &anchor_spl::token::ID,
    );
    let vault_ata_y = get_associated_token_address_with_program_id(
        &vault_pda,
        &USDT_MINT,
        &anchor_spl::token::ID,
    );

    let accounts = dlmm_vault::accounts::Initialize {
        vault_account: vault_pda,
        signer: mock_user.pubkey(),
        token_x_mint: USDC_MINT,
        token_y_mint: USDT_MINT,
        token_x_program: anchor_spl::token::ID,
        token_y_program: anchor_spl::token::ID,
        dlmm_pool: USDC_USDT_POOL.to_bytes().into(),
        system_program: system_program::ID,
        token_x_ata: vault_ata_x,
        token_y_ata: vault_ata_y,
        associated_token_program: anchor_spl::associated_token::ID,
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
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&user]).unwrap();
    let sim_res = svm.simulate_transaction(tx.clone()).unwrap();
    let meta = svm.send_transaction(tx).unwrap();
    assert_eq!(sim_res.meta, meta);
    assert_eq!(meta.logs[1], "Program log: Instruction: Initialize");
    println!("logs: {:#?}", meta.logs);
    assert!(meta.compute_units_consumed < 100_000);

    let data_line = meta
        .logs
        .iter()
        .find(|l| l.starts_with("Program data: "))
        .expect("event not found")
        .trim_start_matches("Program data: ")
        .to_string();

    let raw = decode(data_line).expect("base64 decode");
    let (disc, body) = raw.split_at(8);

    let want_disc = &hashv(&[b"event:", b"InitializeEvent"]).to_bytes()[..8];
    assert_eq!(disc, want_disc, "unexpected event type");

    let ev = InitializeEvent::try_from_slice(body).expect("borsh decode");
    assert_eq!(ev.vault_account, vault_pda);
    assert_eq!(ev.owner, mock_user.pubkey());
    assert_eq!(ev.token_x_mint, USDC_MINT);
    assert_eq!(ev.token_y_mint, USDT_MINT);
    assert_eq!(ev.dlmm_pool, USDC_USDT_POOL);
    assert_eq!(ev.lower_price_range_bps, 0);
    assert_eq!(ev.upper_price_range_bps, 0);
    assert_eq!(ev.operator, mock_user.pubkey());
    assert_eq!(ev.position_id, Pubkey::default());
    println!("ev: {:#?}", ev);
}
