use anchor_lang::AnchorDeserialize;
use dlmm_vault::events::initialize::InitializeEvent;
use litesvm::LiteSVM;
use solana_keypair::{Keypair as SKeypair, Signer as SSigner};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::{signature::Keypair, signer::Signer};

use crate::helpers::account::load_account;
use crate::helpers::event::find_event;
use crate::helpers::initialize_ix::initialize_vault_ix;
use crate::helpers::program::load_dlmm_vault_program;
use crate::helpers::transaction::prepare_tx;

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

    let (initialize_ix, vault_pda, _, _) =
        initialize_vault_ix(&user_clone, &USDC_MINT, &USDT_MINT, &USDC_USDT_POOL);

    let tx = prepare_tx(&mut svm, &user.pubkey(), &[&user], &[initialize_ix]);
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
