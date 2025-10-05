use anchor_lang::AnchorDeserialize;
use dlmm_vault::dlmm::types::{BinLiquidityDistribution, BinLiquidityReduction};
use dlmm_vault::events::add_liquidity::AddLiquidityEvent;
use dlmm_vault::events::claim_fees::ClaimFeesEvent;
use dlmm_vault::events::close_position::ClosePositionEvent;
use dlmm_vault::events::create_position::CreatePositionEvent;
use dlmm_vault::events::remove_liquidity::RemoveLiquidityEvent;
use dlmm_vault::DlmmVaultAccount;
use litesvm::LiteSVM;
use solana_keypair::{Keypair as SKeypair, Signer as SSigner};
use solana_sdk::native_token::LAMPORTS_PER_SOL;
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
use crate::helpers::{
    claim_fees_ix::claim_fees_ix, close_position_ix::close_position_ix,
    remove_liquidity_ix::remove_liquidity_ix,
};

const USDC_USDT_POOL: Pubkey = solana_sdk::pubkey!("ARwi1S4DaiTG5DX7S4M4ZsrXqpMD1MrTmbu9ue2tpmEq");
const USDC_MINT: Pubkey = solana_sdk::pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const USDT_MINT: Pubkey = solana_sdk::pubkey!("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB");
const RENT_PROGRAM: Pubkey = solana_sdk::pubkey!("SysvarRent111111111111111111111111111111111");
const MEMO_PROGRAM: Pubkey = solana_sdk::pubkey!("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr");
#[test]
fn test_close_position() {
    let user = SKeypair::new();
    let user_clone = Keypair::from_bytes(&user.to_bytes()).unwrap();

    let mut svm = LiteSVM::new();
    load_dlmm_vault_program(&mut svm);
    load_dlmm_program(&mut svm);

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

    // TODO: Is this right?
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

    // TODO: Move to own test
    let bin_id = pool_state.active_id;
    let active_bin_array_idx = bin_id_to_bin_array_index(bin_id).unwrap();
    let (bin_array_key, _bump) = derive_bin_array_pda(USDC_USDT_POOL, active_bin_array_idx.into());

    let (top_bin_array_key, _bump) =
        derive_bin_array_pda(USDC_USDT_POOL, (active_bin_array_idx + 1).into());

    let add_liquidity_ix = add_liquidity_ix(
        &user_clone,
        &vault_pda,
        &USDC_USDT_POOL,
        &None,
        &pool_state.reserve_x,
        &pool_state.reserve_y,
        &vault_ata_x,
        &vault_ata_y,
        &USDC_MINT,
        &USDT_MINT,
        &position_pda,
        &anchor_spl::token::ID,
        &anchor_spl::token::ID,
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

    let setup_tx = prepare_tx(
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
    svm.send_transaction(setup_tx).unwrap();

    // We are setup now, so we can do the "close" action which is:
    // - claim fees across the bins
    // - remove liquidity
    // - close position

    let claim_fees_ix = claim_fees_ix(
        &user_clone,
        &vault_pda,
        &USDC_USDT_POOL,
        &pool_state.reserve_x,
        &pool_state.reserve_y,
        &vault_ata_x,
        &vault_ata_y,
        &USDC_MINT,
        &USDT_MINT,
        &position_pda,
        &anchor_spl::token::ID,
        &anchor_spl::token::ID,
        &event_authority_pda,
        &dlmm_vault::dlmm::ID,
        &MEMO_PROGRAM,
        &bin_array_key,
        &top_bin_array_key,
        lower_bin_id,
        bin_id,
    );

    let remove_liquidity_ix = remove_liquidity_ix(
        &user_clone,
        &vault_pda,
        &USDC_USDT_POOL,
        &None,
        &pool_state.reserve_x,
        &pool_state.reserve_y,
        &vault_ata_x,
        &vault_ata_y,
        &USDC_MINT,
        &USDT_MINT,
        &position_pda,
        &anchor_spl::token::ID,
        &anchor_spl::token::ID,
        &event_authority_pda,
        &dlmm_vault::dlmm::ID,
        &MEMO_PROGRAM,
        &bin_array_key,
        &top_bin_array_key,
        vec![
            BinLiquidityReduction {
                bin_id: pool_state.active_id - 1,
                bps_to_remove: 10000,
            },
            BinLiquidityReduction {
                bin_id: pool_state.active_id,
                bps_to_remove: 10000,
            },
            BinLiquidityReduction {
                bin_id: pool_state.active_id + 1,
                bps_to_remove: 10000,
            },
        ],
    );

    let close_position_ix = close_position_ix(
        &user_clone,
        &vault_pda,
        &position_pda,
        &dlmm_vault::dlmm::ID,
        &event_authority_pda,
    );

    // Get the SOL balance of the user before sending the next transaction
    let sol_balance_before_close = svm
        .get_balance(&user_clone.pubkey().to_bytes().into())
        .unwrap();

    let tx = prepare_tx(
        &mut svm,
        &user.pubkey(),
        &[&user],
        &[claim_fees_ix, remove_liquidity_ix, close_position_ix],
    );
    let meta = svm.send_transaction(tx).unwrap();

    let body = find_event(&meta.logs, b"ClaimFeesEvent");
    let ev = ClaimFeesEvent::try_from_slice(body.as_slice()).expect("borsh decode");
    assert_eq!(ev.vault_account, vault_pda);
    assert_eq!(ev.position, position_pda);
    // No fees to claim in this instance, so vault balances will be deposit - liquidity amount
    assert_eq!(ev.initial_x_balance, 9800);
    assert_eq!(ev.initial_y_balance, 4800);
    assert_eq!(ev.final_x_balance, 9800);
    assert_eq!(ev.final_y_balance, 4800);

    let body = find_event(&meta.logs, b"RemoveLiquidityEvent");
    let ev = RemoveLiquidityEvent::try_from_slice(body.as_slice()).expect("borsh decode");
    assert_eq!(ev.vault_account, vault_pda);
    assert_eq!(ev.position, position_pda);
    assert_eq!(
        ev.bin_liquidity_reduction[0].bin_id,
        pool_state.active_id - 1
    );
    assert_eq!(ev.bin_liquidity_reduction[0].bps_to_remove, 10000);
    assert_eq!(ev.bin_liquidity_reduction[1].bin_id, pool_state.active_id);
    assert_eq!(ev.bin_liquidity_reduction[1].bps_to_remove, 10000);
    assert_eq!(
        ev.bin_liquidity_reduction[2].bin_id,
        pool_state.active_id + 1
    );
    assert_eq!(ev.bin_liquidity_reduction[2].bps_to_remove, 10000);

    let body = find_event(&meta.logs, b"ClosePositionEvent");
    let ev = ClosePositionEvent::try_from_slice(body.as_slice()).expect("borsh decode");
    assert_eq!(ev.vault_account, vault_pda);
    assert_eq!(ev.position, position_pda);
    assert_eq!(ev.signer, user_clone.pubkey());

    // Validate the vault token accounts have been credited - as no trades have happened entire balance
    // should be returned to the vault
    // Note: these are hardcoded based on the seed, as withdrawing from an active bin gives you the ratio
    // from that bin, so output will not be identical to the input, but very close
    validate_token_account_balance(&mut svm, &vault_ata_x, 10099);
    validate_token_account_balance(&mut svm, &vault_ata_y, 4899);

    // Validate that the DlmmVault account has been updated to in_position = false and has the correct position_id
    let vault_account = svm.get_account(&vault_pda.to_bytes().into()).unwrap();
    let vault_account_data = DlmmVaultAccount::try_from_slice(&vault_account.data[8..]).unwrap();
    assert_eq!(vault_account_data.in_position, false);
    assert_eq!(vault_account_data.position_id, Pubkey::default());

    let sol_balance_after_close = svm
        .get_balance(&user_clone.pubkey().to_bytes().into())
        .unwrap();
    // The users sol balance should go up by at least 0.04 SOL due to reclaim of rent
    assert!(sol_balance_after_close > sol_balance_before_close + LAMPORTS_PER_SOL / 100 * 4);
}

#[test]
fn test_close_position_with_operator() {
    let user = SKeypair::new();
    let user_clone = Keypair::from_bytes(&user.to_bytes()).unwrap();
    let operator = SKeypair::new();
    let operator_clone = Keypair::from_bytes(&operator.to_bytes()).unwrap();

    let mut svm = LiteSVM::new();
    load_dlmm_vault_program(&mut svm);
    load_dlmm_program(&mut svm);

    svm.airdrop(&user_clone.pubkey().to_bytes().into(), 1_000_000_000)
        .unwrap();
    svm.airdrop(&operator_clone.pubkey().to_bytes().into(), 1_000_000_000)
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
        &operator_clone,
        &USDC_MINT,
        &USDT_MINT,
        &USDC_USDT_POOL,
        &anchor_spl::token::ID,
        &anchor_spl::token::ID,
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
        &USDC_MINT,
        &USDT_MINT,
        &anchor_spl::token::ID,
        &anchor_spl::token::ID,
        token_x_deposit_amount,
        token_y_deposit_amount,
    );

    let deposit_tx = prepare_tx(
        &mut svm,
        &user.pubkey(),
        &[&user],
        &[initialize_ix, deposit_ix],
    );
    svm.send_transaction(deposit_tx).unwrap();

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
        &operator_clone,
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

    // TODO: Move to own test
    let bin_id = pool_state.active_id;
    let active_bin_array_idx = bin_id_to_bin_array_index(bin_id).unwrap();
    let (bin_array_key, _bump) = derive_bin_array_pda(USDC_USDT_POOL, active_bin_array_idx.into());

    let (top_bin_array_key, _bump) =
        derive_bin_array_pda(USDC_USDT_POOL, (active_bin_array_idx + 1).into());

    let add_liquidity_ix = add_liquidity_ix(
        &operator_clone,
        &vault_pda,
        &USDC_USDT_POOL,
        &None,
        &pool_state.reserve_x,
        &pool_state.reserve_y,
        &vault_ata_x,
        &vault_ata_y,
        &USDC_MINT,
        &USDT_MINT,
        &position_pda,
        &anchor_spl::token::ID,
        &anchor_spl::token::ID,
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

    let setup_tx = prepare_tx(
        &mut svm,
        &operator.pubkey(),
        &[&operator],
        &[create_position_ix, add_liquidity_ix],
    );
    svm.send_transaction(setup_tx).unwrap();

    // We are setup now, so we can do the "close" action which is:
    // - claim fees across the bins
    // - remove liquidity
    // - close position

    let claim_fees_ix = claim_fees_ix(
        &operator_clone,
        &vault_pda,
        &USDC_USDT_POOL,
        &pool_state.reserve_x,
        &pool_state.reserve_y,
        &vault_ata_x,
        &vault_ata_y,
        &USDC_MINT,
        &USDT_MINT,
        &position_pda,
        &anchor_spl::token::ID,
        &anchor_spl::token::ID,
        &event_authority_pda,
        &dlmm_vault::dlmm::ID,
        &MEMO_PROGRAM,
        &bin_array_key,
        &top_bin_array_key,
        lower_bin_id,
        bin_id,
    );

    let remove_liquidity_ix = remove_liquidity_ix(
        &operator_clone,
        &vault_pda,
        &USDC_USDT_POOL,
        &None,
        &pool_state.reserve_x,
        &pool_state.reserve_y,
        &vault_ata_x,
        &vault_ata_y,
        &USDC_MINT,
        &USDT_MINT,
        &position_pda,
        &anchor_spl::token::ID,
        &anchor_spl::token::ID,
        &event_authority_pda,
        &dlmm_vault::dlmm::ID,
        &MEMO_PROGRAM,
        &bin_array_key,
        &top_bin_array_key,
        vec![
            BinLiquidityReduction {
                bin_id: pool_state.active_id - 1,
                bps_to_remove: 10000,
            },
            BinLiquidityReduction {
                bin_id: pool_state.active_id,
                bps_to_remove: 10000,
            },
            BinLiquidityReduction {
                bin_id: pool_state.active_id + 1,
                bps_to_remove: 10000,
            },
        ],
    );

    let close_position_ix = close_position_ix(
        &operator_clone,
        &vault_pda,
        &position_pda,
        &dlmm_vault::dlmm::ID,
        &event_authority_pda,
    );

    // Get the SOL balance of the user before sending the next transaction
    let sol_balance_before_close = svm
        .get_balance(&operator_clone.pubkey().to_bytes().into())
        .unwrap();

    let tx = prepare_tx(
        &mut svm,
        &operator.pubkey(),
        &[&operator],
        &[claim_fees_ix, remove_liquidity_ix, close_position_ix],
    );
    let meta = svm.send_transaction(tx).unwrap();

    let body = find_event(&meta.logs, b"ClaimFeesEvent");
    let ev = ClaimFeesEvent::try_from_slice(body.as_slice()).expect("borsh decode");
    assert_eq!(ev.vault_account, vault_pda);
    assert_eq!(ev.position, position_pda);
    // No fees to claim in this instance, so vault balances will be deposit - liquidity amount
    assert_eq!(ev.initial_x_balance, 9800);
    assert_eq!(ev.initial_y_balance, 4800);
    assert_eq!(ev.final_x_balance, 9800);
    assert_eq!(ev.final_y_balance, 4800);

    let body = find_event(&meta.logs, b"RemoveLiquidityEvent");
    let ev = RemoveLiquidityEvent::try_from_slice(body.as_slice()).expect("borsh decode");
    assert_eq!(ev.vault_account, vault_pda);
    assert_eq!(ev.position, position_pda);
    assert_eq!(
        ev.bin_liquidity_reduction[0].bin_id,
        pool_state.active_id - 1
    );
    assert_eq!(ev.bin_liquidity_reduction[0].bps_to_remove, 10000);
    assert_eq!(ev.bin_liquidity_reduction[1].bin_id, pool_state.active_id);
    assert_eq!(ev.bin_liquidity_reduction[1].bps_to_remove, 10000);
    assert_eq!(
        ev.bin_liquidity_reduction[2].bin_id,
        pool_state.active_id + 1
    );
    assert_eq!(ev.bin_liquidity_reduction[2].bps_to_remove, 10000);

    let body = find_event(&meta.logs, b"ClosePositionEvent");
    let ev = ClosePositionEvent::try_from_slice(body.as_slice()).expect("borsh decode");
    assert_eq!(ev.vault_account, vault_pda);
    assert_eq!(ev.position, position_pda);
    assert_eq!(ev.signer, operator_clone.pubkey());

    // Validate the vault token accounts have been credited - as no trades have happened entire balance
    // should be returned to the vault
    // Note: these are hardcoded based on the seed, as withdrawing from an active bin gives you the ratio
    // from that bin, so output will not be identical to the input, but very close
    validate_token_account_balance(&mut svm, &vault_ata_x, 10099);
    validate_token_account_balance(&mut svm, &vault_ata_y, 4899);

    // Validate that the DlmmVault account has been updated to in_position = false and has the correct position_id
    let vault_account = svm.get_account(&vault_pda.to_bytes().into()).unwrap();
    let vault_account_data = DlmmVaultAccount::try_from_slice(&vault_account.data[8..]).unwrap();
    assert_eq!(vault_account_data.in_position, false);
    assert_eq!(vault_account_data.position_id, Pubkey::default());

    let sol_balance_after_close = svm
        .get_balance(&operator_clone.pubkey().to_bytes().into())
        .unwrap();
    // The users sol balance should go up by at least 0.04 SOL due to reclaim of rent
    assert!(sol_balance_after_close > sol_balance_before_close + LAMPORTS_PER_SOL / 100 * 4);
}
