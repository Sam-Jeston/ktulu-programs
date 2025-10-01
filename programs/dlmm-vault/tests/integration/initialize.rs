use anchor_lang::solana_program::hash::hashv;
use anchor_lang::{system_program, AnchorDeserialize, InstructionData, ToAccountMetas};
use dlmm_vault::events::initialize::InitializeEvent;
use litesvm::LiteSVM;
use solana_instruction::account_meta::AccountMeta as SAccountMeta;
use solana_keypair::{Keypair as SKeypair, Signer as SSigner};
use solana_message::Instruction;
use solana_message::{Message, VersionedMessage};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::{signature::Keypair, signer::Signer};
use solana_transaction::versioned::VersionedTransaction;
use spl_associated_token_account::get_associated_token_address_with_program_id;

use crate::helpers::account::load_account;
use crate::helpers::event::find_event;
use crate::helpers::program::load_dlmm_vault_program;

const USDC_USDT_POOL: Pubkey = solana_sdk::pubkey!("ARwi1S4DaiTG5DX7S4M4ZsrXqpMD1MrTmbu9ue2tpmEq");
const USDC_MINT: Pubkey = solana_sdk::pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const USDT_MINT: Pubkey = solana_sdk::pubkey!("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB");

#[test]
fn test_initialize() {
    let user = SKeypair::new();
    let user_clone = Keypair::from_bytes(&user.to_bytes()).unwrap();

    let mut svm = LiteSVM::new();
    load_dlmm_vault_program(&mut svm);

    svm.airdrop(&user_clone.pubkey().to_bytes().into(), 1_000_000_000)
        .unwrap();

    load_account(&mut svm, &USDC_USDT_POOL);
    load_account(&mut svm, &USDC_MINT);
    load_account(&mut svm, &USDT_MINT);

    let ix_data = dlmm_vault::instruction::Initialize {
        token_x_mint: USDC_MINT,
        token_y_mint: USDT_MINT,
        lower_price_range_bps: 0,
        upper_price_range_bps: 0,
        operator: user_clone.pubkey(),
    }
    .data();

    let (vault_pda, _bump) = Pubkey::find_program_address(
        &[
            b"dlmm_vault",
            user_clone.pubkey().as_ref(),
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
        signer: user_clone.pubkey(),
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
        Some(&user_clone.pubkey().to_bytes().into()),
        &blockhash,
    );
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&user]).unwrap();
    let sim_res = svm.simulate_transaction(tx.clone()).unwrap();
    let meta = svm.send_transaction(tx).unwrap();
    assert_eq!(sim_res.meta, meta);
    assert!(meta.compute_units_consumed < 100_000);

    let body = find_event(&meta.logs, b"InitializeEvent");
    let ev = InitializeEvent::try_from_slice(body.as_slice()).expect("borsh decode");
    assert_eq!(ev.vault_account, vault_pda);
    assert_eq!(ev.owner, user_clone.pubkey());
    assert_eq!(ev.token_x_mint, USDC_MINT);
    assert_eq!(ev.token_y_mint, USDT_MINT);
    assert_eq!(ev.dlmm_pool, USDC_USDT_POOL);
    assert_eq!(ev.lower_price_range_bps, 0);
    assert_eq!(ev.upper_price_range_bps, 0);
    assert_eq!(ev.operator, user_clone.pubkey());
    assert_eq!(ev.position_id, Pubkey::default());
}
