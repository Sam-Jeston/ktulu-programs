use anchor_lang::AnchorDeserialize;
use dlmm_vault::events::deposit::DepositEvent;
use litesvm::LiteSVM;
use solana_keypair::{Keypair as SKeypair, Signer as SSigner};
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::{signature::Keypair, signer::Signer};

use crate::helpers::account::load_account;
use crate::helpers::close_ix::close_ix;
use crate::helpers::deposit_ix::deposit_vault_ix;
use crate::helpers::event::find_event;
use crate::helpers::initialize_ix::initialize_vault_ix;
use crate::helpers::log::assert_logs_contain;
use crate::helpers::program::load_dlmm_vault_program;
use crate::helpers::token::{create_and_fund_token_account, validate_token_account_balance};
use crate::helpers::transaction::prepare_tx;

const USDC_USDT_POOL: Pubkey = solana_sdk::pubkey!("ARwi1S4DaiTG5DX7S4M4ZsrXqpMD1MrTmbu9ue2tpmEq");
const USDC_MINT: Pubkey = solana_sdk::pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const USDT_MINT: Pubkey = solana_sdk::pubkey!("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB");

#[test]
fn test_close() {
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
        &anchor_spl::token::ID,
    );
    let user_ata_y = create_and_fund_token_account(
        &mut svm,
        &user_clone.pubkey(),
        &USDT_MINT,
        token_y_initial_balance,
        &anchor_spl::token::ID,
    );

    let (initialize_ix, vault_pda, vault_ata_x, vault_ata_y) = initialize_vault_ix(
        &user_clone,
        &user_clone,
        &USDC_MINT,
        &USDT_MINT,
        &USDC_USDT_POOL,
        &anchor_spl::token::ID,
        &anchor_spl::token::ID,
    );

    // let token_x_deposit_amount = 10_000;
    // let token_y_deposit_amount = 5_000;

    // let deposit_ix = deposit_vault_ix(
    //     &user_clone,
    //     &vault_pda,
    //     &user_ata_x,
    //     &vault_ata_x,
    //     &user_ata_y,
    //     &vault_ata_y,
    //     &USDC_MINT,
    //     &USDT_MINT,
    //     &anchor_spl::token::ID,
    //     &anchor_spl::token::ID,
    //     token_x_deposit_amount,
    //     token_y_deposit_amount,
    // );

    let setup_tx = prepare_tx(&mut svm, &user.pubkey(), &[&user], &[initialize_ix]);
    svm.send_transaction(setup_tx).unwrap();

    let close_ix = close_ix(
        &vault_pda,
        &user_clone.pubkey(),
        &vault_ata_x,
        &vault_ata_y,
        &USDC_MINT,
        &USDT_MINT,
        &anchor_spl::token::ID,
        &anchor_spl::token::ID,
        &user_ata_x,
        &user_ata_y,
    );

    let sol_balance_before_close = svm
        .get_balance(&user_clone.pubkey().to_bytes().into())
        .unwrap();

    let close_tx = prepare_tx(&mut svm, &user.pubkey(), &[&user], &[close_ix]);
    let meta = svm.send_transaction(close_tx).unwrap();

    let sol_balance_after_close = svm
        .get_balance(&user_clone.pubkey().to_bytes().into())
        .unwrap();
    // The users sol balance should go up by at least 0.006 SOL due to token program rent for vault atas, and the vault data
    assert!(sol_balance_after_close > sol_balance_before_close + LAMPORTS_PER_SOL / 1000 * 6);
}
