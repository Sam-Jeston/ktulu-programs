use anchor_lang::{AnchorDeserialize, Id, InstructionData, ToAccountMetas};
use dlmm_vault::events::rebalance::RebalanceEvent;
use dlmm_vault::{FeeCompoundingStrategy, VolatilityStrategy};
use litesvm::LiteSVM;
use solana_compute_budget::compute_budget::ComputeBudget;
use solana_instruction::account_meta::AccountMeta as SAccountMeta;
use solana_keypair::{Keypair as SKeypair, Signer as SSigner};
use solana_message::Instruction;
use solana_program_test::tokio;
use solana_sdk::program_pack::Pack;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::{signature::Keypair, signer::Signer};
use spl_token::state::{Account as TokenAccount, AccountState};

use crate::helpers::account::load_account;
use crate::helpers::atl::get_address_lookup_table_accounts;
use crate::helpers::deposit_ix::deposit_vault_ix;
use crate::helpers::event::find_event;
use crate::helpers::initialize_ix::initialize_vault_ix;
use crate::helpers::program::{
    load_dlmm_program, load_dlmm_vault_program, load_jupiter_program, load_whirlpool_program,
};
use crate::helpers::token::create_and_fund_token_account;
use crate::helpers::transaction::{prepare_tx, prepare_v0_tx};

use jup_swap::{
    quote::QuoteRequest,
    swap::SwapRequest,
    transaction_config::{DynamicSlippageSettings, TransactionConfig},
    JupiterSwapApiClient,
};

const USDC_USDT_POOL: Pubkey = solana_sdk::pubkey!("ARwi1S4DaiTG5DX7S4M4ZsrXqpMD1MrTmbu9ue2tpmEq");
const USDC_MINT: Pubkey = solana_sdk::pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const USDT_MINT: Pubkey = solana_sdk::pubkey!("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB");

const TOKEN2022_PROGRAM: Pubkey =
    solana_sdk::pubkey!("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");
const JUPITER_PROGRAM: Pubkey = solana_sdk::pubkey!("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4");
const DLMM_PROGRAM: Pubkey = solana_sdk::pubkey!("LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo");

#[tokio::test]
async fn test_rebalance() {
    let user = SKeypair::new();
    let user_clone = Keypair::from_bytes(&user.to_bytes()).unwrap();
    let operator = SKeypair::new();
    let operator_clone = Keypair::from_bytes(&operator.to_bytes()).unwrap();

    let mut svm = LiteSVM::new();
    load_dlmm_vault_program(&mut svm);
    load_jupiter_program(&mut svm);
    load_dlmm_program(&mut svm);
    load_whirlpool_program(&mut svm);

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
    let operator_ata_y = create_and_fund_token_account(
        &mut svm,
        &operator_clone.pubkey(),
        &USDT_MINT,
        token_y_initial_balance,
        &anchor_spl::token::ID,
    );

    let (initialize_ix, vault_pda, vault_ata_x, vault_ata_y, _) = initialize_vault_ix(
        &user_clone,
        &operator_clone,
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

    let token_x_deposit_amount = 100_000;
    let token_y_deposit_amount = 1_000;

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
    svm.send_transaction(tx).unwrap();

    // Now that the vault is funded, we can rebalance
    let api_base_url = "https://lite-api.jup.ag/swap/v1";
    let jupiter_swap_api_client = JupiterSwapApiClient::new(api_base_url.into());

    let input_amount = 45_000;

    let quote_request = QuoteRequest {
        amount: input_amount,
        input_mint: USDC_MINT,
        output_mint: USDT_MINT,
        platform_fee_bps: Some(10),
        // Limit quoting into DLMM for sake of testing
        dexes: Some("Meteora DLMM".to_string()),
        ..QuoteRequest::default()
    };

    // Uncomment for fresh quote
    // let quote_response = match jupiter_swap_api_client.quote(&quote_request).await {
    //     Ok(quote_response) => quote_response,
    //     Err(e) => {
    //         println!("quote failed: {e:#?}");
    //         return;
    //     }
    // };

    // let quote_response_json = serde_json::to_string(&quote_response).unwrap();
    // println!("quote response JSON: {}", quote_response_json);

    let quote_response_json = "{\"inputMint\":\"EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v\",\"inAmount\":\"45000\",\"outputMint\":\"Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB\",\"outAmount\":\"44906\",\"otherAmountThreshold\":\"44906\",\"swapMode\":\"ExactIn\",\"slippageBps\":0,\"platformFee\":{\"amount\":\"44\",\"feeBps\":10},\"priceImpactPct\":\"0.0003233659234587966428656893\",\"routePlan\":[{\"swapInfo\":{\"ammKey\":\"4bg8UDLXEm4T6pCyoW7iUizAz9HMoxhTAtMquSXigFZu\",\"label\":\"Meteora DLMM\",\"inputMint\":\"EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v\",\"outputMint\":\"Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB\",\"inAmount\":\"45000\",\"outAmount\":\"44950\",\"feeAmount\":\"6\",\"feeMint\":\"EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v\"},\"percent\":100}],\"contextSlot\":373228081,\"timeTaken\":0.000506802}";
    let quote_response = serde_json::from_str(&quote_response_json).unwrap();

    let response = jupiter_swap_api_client
        .swap_instructions(&SwapRequest {
            user_public_key: vault_pda,
            quote_response,
            config: TransactionConfig {
                skip_user_accounts_rpc_calls: true,
                wrap_and_unwrap_sol: false,
                fee_account: Some(operator_ata_y.clone()),
                dynamic_compute_unit_limit: true,
                dynamic_slippage: Some(DynamicSlippageSettings {
                    min_bps: Some(50),
                    max_bps: Some(1000),
                }),
                ..TransactionConfig::default()
            },
        })
        .await
        .unwrap();

    let address_lookup_table_accounts =
        get_address_lookup_table_accounts(&mut svm, response.address_lookup_table_addresses)
            .unwrap();

    let ix_data = dlmm_vault::instruction::HandleRebalance {
        rebalance_data: response.swap_instruction.data,
    }
    .data();

    let mut accounts = dlmm_vault::accounts::Rebalance {
        vault_account: vault_pda.clone(),
        signer: user_clone.pubkey(),
        input_mint: USDC_MINT.clone(),
        vault_input_token_account: vault_ata_x.clone(),
        input_token_program: anchor_spl::token::ID.clone(),
        output_mint: USDT_MINT.clone(),
        vault_output_token_account: vault_ata_y.clone(),
        output_token_program: anchor_spl::token::ID.clone(),
        operator_fee_account: operator_ata_y.clone(),
        jupiter_program: dlmm_vault::jupiter::program::Jupiter::id()
            .to_bytes()
            .into(),
    }
    .to_account_metas(None);

    let remaining_accounts = response.swap_instruction.accounts;
    accounts.extend(remaining_accounts.into_iter().map(|mut account| {
        // Load the account unless its the token, token2022 or jupiter program key
        if account.pubkey != anchor_spl::token::ID
            && account.pubkey != TOKEN2022_PROGRAM
            && account.pubkey != JUPITER_PROGRAM
            && account.pubkey != vault_ata_x
            && account.pubkey != vault_ata_y
            && account.pubkey != operator_ata_y
            && account.pubkey != USDC_MINT
            && account.pubkey != USDT_MINT
            && account.pubkey != vault_pda
            && account.pubkey != DLMM_PROGRAM
        {
            load_account(&mut svm, &account.pubkey);
        }
        account.is_signer = false;
        account
    }));

    let rebalance_ix = Instruction {
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

    let mut compute_budget = ComputeBudget::new_with_defaults(true);
    compute_budget.compute_unit_limit = 400_000;
    svm = svm.with_compute_budget(compute_budget);

    let tx = prepare_v0_tx(
        &mut svm,
        &user.pubkey(),
        &[&user],
        &address_lookup_table_accounts,
        &[rebalance_ix],
    );
    let meta = svm.send_transaction(tx).unwrap();

    // Ensure comput units used is less than 300_000
    assert!(meta.compute_units_consumed < 300_000);

    // Print the vault token balances after the swap
    let token_account_in = svm.get_account(&vault_ata_x.to_bytes().into()).unwrap();
    let token_account_data_in = TokenAccount::unpack_from_slice(&token_account_in.data).unwrap();
    let token_account_out = svm.get_account(&vault_ata_y.to_bytes().into()).unwrap();
    let token_account_data_out = TokenAccount::unpack_from_slice(&token_account_out.data).unwrap();

    let body = find_event(&meta.logs, b"RebalanceEvent");
    let ev = RebalanceEvent::try_from_slice(body.as_slice()).expect("failed to borsh decode");
    assert_eq!(ev.vault_account, vault_pda);
    assert_eq!(ev.in_mint, USDC_MINT);
    assert_eq!(ev.out_mint, USDT_MINT);
    assert_eq!(ev.initial_in_balance, token_x_deposit_amount);
    assert_eq!(ev.initial_out_balance, token_y_deposit_amount);
    assert_eq!(ev.final_in_balance, token_account_data_in.amount);
    assert_eq!(ev.final_out_balance, token_account_data_out.amount);
    assert_eq!(ev.signer, user_clone.pubkey());
}
