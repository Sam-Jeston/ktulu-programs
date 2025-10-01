use anchor_lang::AnchorDeserialize;
use litesvm::LiteSVM;
use solana_keypair::{Keypair as SKeypair, Signer as SSigner};
use solana_sdk::program_pack::Pack;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::{signature::Keypair, signer::Signer};
use spl_token::state::Account as TokenAccount;

use crate::helpers::account::load_account;
use crate::helpers::deposit_ix::deposit_vault_ix;
use crate::helpers::initialize_ix::initialize_vault_ix;
use crate::helpers::program::load_dlmm_vault_program;
use crate::helpers::token::{create_and_fund_token_account, validate_token_account_balance};
use crate::helpers::transaction::prepare_tx;

const USDC_USDT_POOL: Pubkey = solana_sdk::pubkey!("ARwi1S4DaiTG5DX7S4M4ZsrXqpMD1MrTmbu9ue2tpmEq");
const USDC_MINT: Pubkey = solana_sdk::pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const USDT_MINT: Pubkey = solana_sdk::pubkey!("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB");

#[test]
fn test_deposit() {
    let user = SKeypair::new();
    let user_clone = Keypair::from_bytes(&user.to_bytes()).unwrap();

    let mut svm = LiteSVM::new();
    load_dlmm_vault_program(&mut svm);

    svm.airdrop(&user_clone.pubkey().to_bytes().into(), 1_000_000_000)
        .unwrap();

    load_account(&mut svm, &USDC_USDT_POOL);
    load_account(&mut svm, &USDC_MINT);
    load_account(&mut svm, &USDT_MINT);
    let user_ata_x =
        create_and_fund_token_account(&mut svm, &user_clone, &USDC_MINT, 1_000_000_000);
    let user_ata_y =
        create_and_fund_token_account(&mut svm, &user_clone, &USDT_MINT, 1_000_000_000);

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

    let tx = prepare_tx(
        &mut svm,
        &user.pubkey(),
        &[&user],
        &[initialize_ix, deposit_ix],
    );
    let sim_res = svm.simulate_transaction(tx.clone()).unwrap();
    let meta = svm.send_transaction(tx).unwrap();
    assert_eq!(sim_res.meta, meta);
    assert!(meta.compute_units_consumed < 200_000);

    // Validate that the token account balances have changes as expected
    validate_token_account_balance(&mut svm, &vault_ata_x, token_x_deposit_amount);
    validate_token_account_balance(&mut svm, &vault_ata_y, token_y_deposit_amount);

    // TODO: Validate source vault is debited as expected
}
