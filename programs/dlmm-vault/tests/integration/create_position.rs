use anchor_lang::AnchorDeserialize;
use dlmm_vault::events::create_position::CreatePositionEvent;
use dlmm_vault::events::deposit::DepositEvent;
use litesvm::LiteSVM;
use solana_keypair::{Keypair as SKeypair, Signer as SSigner};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::{signature::Keypair, signer::Signer};

use crate::helpers::account::load_account;
use crate::helpers::create_position_ix::create_position_ix;
use crate::helpers::deposit_ix::deposit_vault_ix;
use crate::helpers::dlmm::load_dlmm_accounts;
use crate::helpers::dlmm_pda::{derive_event_authority_pda, derive_position_pda};
use crate::helpers::event::find_event;
use crate::helpers::initialize_ix::initialize_vault_ix;
use crate::helpers::log::assert_logs_contain;
use crate::helpers::program::load_dlmm_vault_program;
use crate::helpers::token::{create_and_fund_token_account, validate_token_account_balance};
use crate::helpers::transaction::prepare_tx;

const USDC_USDT_POOL: Pubkey = solana_sdk::pubkey!("ARwi1S4DaiTG5DX7S4M4ZsrXqpMD1MrTmbu9ue2tpmEq");
const USDC_MINT: Pubkey = solana_sdk::pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const USDT_MINT: Pubkey = solana_sdk::pubkey!("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB");
const RENT_PROGRAM: Pubkey = solana_sdk::pubkey!("SysvarRent111111111111111111111111111111111");
#[test]
fn test_create_position() {
    let user = SKeypair::new();
    let user_clone = Keypair::from_bytes(&user.to_bytes()).unwrap();

    let mut svm = LiteSVM::new();
    load_dlmm_vault_program(&mut svm);

    svm.airdrop(&user_clone.pubkey().to_bytes().into(), 1_000_000_000)
        .unwrap();

    load_account(&mut svm, &USDC_USDT_POOL);
    load_account(&mut svm, &USDC_MINT);
    load_account(&mut svm, &USDT_MINT);

    let token_x_initial_balance = 1_000_000_000;
    let token_y_initial_balance = 1_000_000_000;

    let user_ata_x = create_and_fund_token_account(
        &mut svm,
        &user_clone.pubkey(),
        &USDC_MINT,
        token_x_initial_balance,
    );
    let user_ata_y = create_and_fund_token_account(
        &mut svm,
        &user_clone.pubkey(),
        &USDT_MINT,
        token_y_initial_balance,
    );

    let (initialize_ix, vault_pda, vault_ata_x, vault_ata_y) =
        initialize_vault_ix(&user_clone, &USDC_MINT, &USDT_MINT, &USDC_USDT_POOL);

    let token_x_deposit_amount = 10_000;
    let token_y_deposit_amount = 5_000;

    let deposit_ix = deposit_vault_ix(
        &user_clone,
        &vault_pda,
        &user_ata_x,
        &vault_ata_x,
        &user_ata_y,
        &vault_ata_y,
        &USDC_MINT,
        &USDT_MINT,
        &anchor_spl::token::ID,
        &anchor_spl::token::ID,
        token_x_deposit_amount,
        token_y_deposit_amount,
    );

    let pool_state = load_dlmm_accounts(&mut svm, &USDC_USDT_POOL).unwrap();

    let lower_bin_id = pool_state.active_id - 3;
    let width = 5;

    let (position_pda, _bump) = derive_position_pda(
        USDC_USDT_POOL.to_bytes().into(),
        vault_pda.to_bytes().into(),
        lower_bin_id,
        width,
    );

    let (event_authority_pda, _bump) = derive_event_authority_pda();

    let create_position_ix = create_position_ix(
        &user_clone,
        &vault_pda,
        &USDC_USDT_POOL,
        &position_pda,
        &dlmm_vault::dlmm::ID,
        &event_authority_pda,
        &RENT_PROGRAM,
        &anchor_lang::system_program::ID,
        lower_bin_id,
        width,
    );

    let tx = prepare_tx(
        &mut svm,
        &user.pubkey(),
        &[&user],
        &[initialize_ix, deposit_ix, create_position_ix],
    );
    let sim_res = svm.simulate_transaction(tx.clone()).unwrap();
    let meta = svm.send_transaction(tx).unwrap();
    assert_eq!(sim_res.meta, meta);
    assert!(meta.compute_units_consumed < 200_000);

    let body = find_event(&meta.logs, b"CreatePositionEvent");
    let ev = CreatePositionEvent::try_from_slice(body.as_slice()).expect("borsh decode");
    assert_eq!(ev.vault_account, vault_pda);
    assert_eq!(ev.position, position_pda);
    assert_eq!(ev.lower_bin_id, -10);
    assert_eq!(ev.width, 5);
}
