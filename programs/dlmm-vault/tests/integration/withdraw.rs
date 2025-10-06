use anchor_lang::AnchorDeserialize;
use dlmm_vault::events::deposit::DepositEvent;
use dlmm_vault::events::withdraw::WithdrawEvent;
use dlmm_vault::{FeeCompoundingStrategy, VolatilityStrategy};
use litesvm::LiteSVM;
use solana_keypair::{Keypair as SKeypair, Signer as SSigner};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::{signature::Keypair, signer::Signer};

use crate::helpers::account::load_account;
use crate::helpers::deposit_ix::deposit_vault_ix;
use crate::helpers::event::find_event;
use crate::helpers::initialize_ix::initialize_vault_ix;
use crate::helpers::program::load_dlmm_vault_program;
use crate::helpers::token::{
    create_and_fund_token_account, create_and_fund_token_account_by_pubkey,
    validate_token_account_balance,
};
use crate::helpers::transaction::prepare_tx;
use crate::helpers::withdraw_ix::withdraw_vault_ix;

const USDC_USDT_POOL: Pubkey = solana_sdk::pubkey!("ARwi1S4DaiTG5DX7S4M4ZsrXqpMD1MrTmbu9ue2tpmEq");
const USDC_MINT: Pubkey = solana_sdk::pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const USDT_MINT: Pubkey = solana_sdk::pubkey!("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB");

#[test]
fn test_withdraw() {
    let user = SKeypair::new();
    let user_clone = Keypair::from_bytes(&user.to_bytes()).unwrap();

    let mut svm = LiteSVM::new();
    load_dlmm_vault_program(&mut svm);

    svm.airdrop(&user_clone.pubkey().to_bytes().into(), 1_000_000_000)
        .unwrap();

    load_account(&mut svm, &USDC_USDT_POOL);
    load_account(&mut svm, &USDC_MINT);
    load_account(&mut svm, &USDT_MINT);

    let token_x_initial_balance = 10;
    let token_y_initial_balance = 20;

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

    let (initialize_ix, vault_pda, vault_ata_x, vault_ata_y, harvest_pda) = initialize_vault_ix(
        &user_clone,
        &user_clone,
        &USDC_MINT,
        &USDT_MINT,
        &USDC_USDT_POOL,
        &anchor_spl::token::ID,
        &anchor_spl::token::ID,
        true,
        true,
        FeeCompoundingStrategy::Aggressive,
        VolatilityStrategy::Spot,
        5,
        false,
        0,
        &USDC_MINT,
        &anchor_spl::token::ID,
    );

    // Submit our init tx to initialize the vault account
    let init_tx = prepare_tx(&mut svm, &user.pubkey(), &[&user], &[initialize_ix]);
    svm.send_transaction(init_tx).unwrap();

    // Fund vault ATAs and execute withdrawal
    let token_x_vault_balance = 11_000;
    let token_y_vault_balance = 21_000;
    create_and_fund_token_account(
        &mut svm,
        &vault_pda,
        &USDC_MINT,
        token_x_vault_balance,
        &anchor_spl::token::ID,
    );
    create_and_fund_token_account(
        &mut svm,
        &vault_pda,
        &USDT_MINT,
        token_y_vault_balance,
        &anchor_spl::token::ID,
    );

    // Fund harvest mint account
    let harvest_mint_balance = 2_000;
    create_and_fund_token_account_by_pubkey(
        &mut svm,
        &vault_pda,
        &USDC_MINT,
        &harvest_pda,
        harvest_mint_balance,
        &anchor_spl::token::ID,
    );

    let token_x_withdraw_amount = 10_000;
    let token_y_withdraw_amount = 5_000;
    let harvest_mint_withdraw_amount = 2_000;

    let withdraw_ix = withdraw_vault_ix(
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
        token_x_withdraw_amount,
        token_y_withdraw_amount,
        &user_ata_x,
        &harvest_pda,
        &USDC_MINT,
        &anchor_spl::token::ID,
        harvest_mint_withdraw_amount,
    );

    let tx = prepare_tx(&mut svm, &user.pubkey(), &[&user], &[withdraw_ix]);
    let sim_res = svm.simulate_transaction(tx.clone()).unwrap();
    let meta = svm.send_transaction(tx).unwrap();
    assert_eq!(sim_res.meta, meta);
    assert!(meta.compute_units_consumed < 100_000);

    // Validate that the vault token account balances have changes as expected
    validate_token_account_balance(
        &mut svm,
        &vault_ata_x,
        token_x_vault_balance - token_x_withdraw_amount,
    );
    validate_token_account_balance(
        &mut svm,
        &vault_ata_y,
        token_y_vault_balance - token_y_withdraw_amount,
    );

    // Validate user accounts are credited as expected
    validate_token_account_balance(
        &mut svm,
        &user_ata_x,
        token_x_initial_balance + token_x_withdraw_amount,
    );
    validate_token_account_balance(
        &mut svm,
        &user_ata_y,
        token_y_initial_balance + token_y_withdraw_amount,
    );

    let body = find_event(&meta.logs, b"WithdrawEvent");
    let ev = WithdrawEvent::try_from_slice(body.as_slice()).expect("borsh decode");
    assert_eq!(ev.vault_account, vault_pda);
    // In this case, the harvest mint is the token x mint, so they get withdrawn to the same destination
    assert_eq!(
        ev.token_x_withdraw_amount,
        token_x_withdraw_amount + harvest_mint_withdraw_amount
    );
    assert_eq!(ev.token_y_withdraw_amount, token_y_withdraw_amount);
}
