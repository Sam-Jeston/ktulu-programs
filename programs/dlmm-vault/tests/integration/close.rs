use dlmm_vault::{FeeCompoundingStrategy, VolatilityStrategy};
use litesvm::LiteSVM;
use solana_keypair::{Keypair as SKeypair, Signer as SSigner};
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::{signature::Keypair, signer::Signer};

use crate::helpers::account::load_account;
use crate::helpers::close_ix::close_ix;
use crate::helpers::initialize_ix::initialize_vault_ix;
use crate::helpers::program::load_dlmm_vault_program;
use crate::helpers::transaction::prepare_tx;

const USDC_USDT_POOL: Pubkey = solana_sdk::pubkey!("ARwi1S4DaiTG5DX7S4M4ZsrXqpMD1MrTmbu9ue2tpmEq");
const USDC_MINT: Pubkey = solana_sdk::pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const USDT_MINT: Pubkey = solana_sdk::pubkey!("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB");
const WSOL_MINT: Pubkey = solana_sdk::pubkey!("So11111111111111111111111111111111111111112");

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
    load_account(&mut svm, &WSOL_MINT);

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
        true,
        0,
        &WSOL_MINT,
        &anchor_spl::token::ID,
    );

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
        &harvest_pda,
        &WSOL_MINT,
        &anchor_spl::token::ID,
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

    // Validate that the token accounts are in fact closed
    assert!(svm.get_account(&vault_ata_x.to_bytes().into()).is_none());
    assert!(svm.get_account(&vault_ata_y.to_bytes().into()).is_none());
    assert!(svm.get_account(&harvest_pda.to_bytes().into()).is_none());
}
