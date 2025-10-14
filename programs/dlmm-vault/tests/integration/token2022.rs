use anchor_lang::AnchorDeserialize;
use dlmm_vault::dlmm::types::BinLiquidityDistribution;
use dlmm_vault::events::add_liquidity::AddLiquidityEvent;
use dlmm_vault::events::create_position::CreatePositionEvent;
use dlmm_vault::{FeeCompoundingStrategy, VolatilityStrategy};
use litesvm::LiteSVM;
use solana_keypair::{Keypair as SKeypair, Signer as SSigner};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::{signature::Keypair, signer::Signer};

use crate::helpers::account::load_account;
use crate::helpers::add_liquidity_ix::add_liquidity_ix;
use crate::helpers::create_position_ix::create_position_ix;
use crate::helpers::deposit_ix::deposit_vault_ix;
use crate::helpers::dlmm::{bin_id_to_bin_array_index, load_dlmm_accounts};
use crate::helpers::dlmm_pda::{
    derive_bin_array_pda, derive_event_authority_pda, derive_position_pda,
};
use crate::helpers::event::find_event;
use crate::helpers::initialize_ix::initialize_vault_ix;
use crate::helpers::program::{load_dlmm_program, load_dlmm_vault_program};
use crate::helpers::token::{create_and_fund_token_account, validate_token_account_balance};
use crate::helpers::transaction::prepare_tx;

const PUMP_USDC_POOL: Pubkey = solana_sdk::pubkey!("9SMp4yLKGtW9TnLimfVPkDARsyNSfJw43WMke4r7KoZj");
const PUMP_MINT: Pubkey = solana_sdk::pubkey!("pumpCmXqMfrsAkQ5r49WcJnRayYRqmXz6ae8H7H9Dfn");
const USDC_MINT: Pubkey = solana_sdk::pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const RENT_PROGRAM: Pubkey = solana_sdk::pubkey!("SysvarRent111111111111111111111111111111111");
const TOKEN2022_PROGRAM: Pubkey =
    solana_sdk::pubkey!("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");
#[test]
fn test_token2022_integration() {
    let user = SKeypair::new();
    let user_clone = Keypair::from_bytes(&user.to_bytes()).unwrap();

    let mut svm = LiteSVM::new();
    load_dlmm_vault_program(&mut svm);
    load_dlmm_program(&mut svm);

    svm.airdrop(&user_clone.pubkey().to_bytes().into(), 1_000_000_000)
        .unwrap();

    load_account(&mut svm, &PUMP_USDC_POOL);
    load_account(&mut svm, &PUMP_MINT);
    load_account(&mut svm, &USDC_MINT);

    let token_x_initial_balance = 1_000_000_000;
    let token_y_initial_balance = 1_000_000_000;

    let token_x_program = TOKEN2022_PROGRAM;
    let token_y_program = anchor_spl::token::ID;

    let user_ata_x = create_and_fund_token_account(
        &mut svm,
        &user_clone.pubkey(),
        &PUMP_MINT,
        token_x_initial_balance,
        &token_x_program,
    );
    let user_ata_y = create_and_fund_token_account(
        &mut svm,
        &user_clone.pubkey(),
        &USDC_MINT,
        token_y_initial_balance,
        &token_y_program,
    );

    let (initialize_ix, vault_pda, vault_ata_x, vault_ata_y, _) = initialize_vault_ix(
        &user_clone,
        &user_clone,
        &PUMP_MINT,
        &USDC_MINT,
        &PUMP_USDC_POOL,
        &token_x_program,
        &token_y_program,
        true,
        true,
        FeeCompoundingStrategy::Aggressive,
        VolatilityStrategy::Spot,
        5,
        false,
        0,
        &PUMP_MINT,
        &token_x_program,
        0,
        0,
        &user_ata_x,
        &user_ata_y,
    );

    let token_x_deposit_amount = 10_000;
    let token_y_deposit_amount = 5_000;

    let deposit_ix = deposit_vault_ix(
        &user_clone,
        &vault_pda,
        &user_ata_x,
        &vault_ata_x,
        &user_ata_y,
        &vault_ata_y,
        &PUMP_MINT,
        &USDC_MINT,
        &token_x_program,
        &token_y_program,
        token_x_deposit_amount,
        token_y_deposit_amount,
    );

    let pool_state = load_dlmm_accounts(&mut svm, &PUMP_USDC_POOL).unwrap();

    let lower_bin_id = pool_state.active_id - 3;
    let width = 5;

    // TODO: Is this right?
    let (position_pda, _bump) = derive_position_pda(
        PUMP_USDC_POOL.to_bytes().into(),
        vault_pda.to_bytes().into(),
        lower_bin_id,
        width,
    );

    let (event_authority_pda, _bump) = derive_event_authority_pda();

    let create_position_ix = create_position_ix(
        &user_clone,
        &vault_pda,
        &PUMP_USDC_POOL,
        &position_pda,
        &dlmm_vault::dlmm::ID,
        &event_authority_pda,
        &RENT_PROGRAM,
        &anchor_lang::system_program::ID,
        lower_bin_id,
        width,
    );

    let bin_id = pool_state.active_id;
    let active_bin_array_idx = bin_id_to_bin_array_index(bin_id).unwrap();
    let (bin_array_key, _bump) = derive_bin_array_pda(PUMP_USDC_POOL, active_bin_array_idx.into());

    let (top_bin_array_key, _bump) =
        derive_bin_array_pda(PUMP_USDC_POOL, (active_bin_array_idx + 1).into());

    let add_liquidity_ix = add_liquidity_ix(
        &user_clone,
        &vault_pda,
        &PUMP_USDC_POOL,
        &None,
        &pool_state.reserve_x,
        &pool_state.reserve_y,
        &vault_ata_x,
        &vault_ata_y,
        &PUMP_MINT,
        &USDC_MINT,
        &position_pda,
        &token_x_program,
        &token_y_program,
        &event_authority_pda,
        &dlmm_vault::dlmm::ID,
        &bin_array_key,
        &top_bin_array_key,
        200,
        200,
        vec![
            BinLiquidityDistribution {
                bin_id: pool_state.active_id - 1,
                distribution_x: 0,
                distribution_y: 5000,
            },
            BinLiquidityDistribution {
                bin_id: pool_state.active_id,
                distribution_x: 5000,
                distribution_y: 5000,
            },
            BinLiquidityDistribution {
                bin_id: pool_state.active_id + 1,
                distribution_x: 5000,
                distribution_y: 0,
            },
        ],
    );

    let tx = prepare_tx(
        &mut svm,
        &user.pubkey(),
        &[&user],
        &[
            initialize_ix,
            deposit_ix,
            create_position_ix,
            add_liquidity_ix,
        ],
    );
    let meta = svm.send_transaction(tx).unwrap();
    assert!(meta.compute_units_consumed < 300_000);

    // Print the user lamports balance after the first tx
    println!(
        "User lamports balance after first tx: {}",
        svm.get_account(&user.pubkey()).unwrap().lamports
    );

    let body = find_event(&meta.logs, b"CreatePositionEvent");
    let ev = CreatePositionEvent::try_from_slice(body.as_slice()).expect("borsh decode");
    assert_eq!(ev.vault_account, vault_pda);
    assert_eq!(ev.position, position_pda);
    assert_eq!(ev.lower_bin_id, -2492);
    assert_eq!(ev.width, 5);

    let body = find_event(&meta.logs, b"AddLiquidityEvent");
    let ev = AddLiquidityEvent::try_from_slice(body.as_slice()).expect("borsh decode");
    assert_eq!(ev.vault_account, vault_pda);
    assert_eq!(ev.token_x_amount, 200);
    assert_eq!(ev.token_y_amount, 200);
    assert_eq!(ev.bin_liquidity_dist[0].bin_id, pool_state.active_id - 1);
    assert_eq!(ev.bin_liquidity_dist[0].distribution_x, 0);
    assert_eq!(ev.bin_liquidity_dist[0].distribution_y, 5000);
    assert_eq!(ev.bin_liquidity_dist[1].bin_id, pool_state.active_id);
    assert_eq!(ev.bin_liquidity_dist[1].distribution_x, 5000);
    assert_eq!(ev.bin_liquidity_dist[1].distribution_y, 5000);
    assert_eq!(ev.bin_liquidity_dist[2].bin_id, pool_state.active_id + 1);
    assert_eq!(ev.bin_liquidity_dist[2].distribution_x, 5000);
    assert_eq!(ev.bin_liquidity_dist[2].distribution_y, 0);

    // Validate the vault token accounts have been debited
    validate_token_account_balance(&mut svm, &vault_ata_x, token_x_deposit_amount - 200);
    validate_token_account_balance(&mut svm, &vault_ata_y, token_y_deposit_amount - 200);
}
