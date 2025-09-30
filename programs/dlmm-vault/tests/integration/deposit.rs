use crate::helpers;
use anchor_lang::{InstructionData, ToAccountMetas, solana_program::pubkey::Pubkey};
use dlmm_vault::dlmm;
use helpers::dlmm_pda::*;
use helpers::dlmm_utils::*;
use helpers::{process_and_assert_ok, setup_dlmm_vault_program};
use solana_program_test::*;
use solana_sdk::account::Account;
use solana_sdk::instruction::AccountMeta;
use solana_sdk::program_option::COption;
use solana_sdk::program_pack::Pack;
use solana_sdk::{
    compute_budget::ComputeBudgetInstruction, instruction::Instruction, signature::Keypair,
    signer::Signer,
};
use spl_associated_token_account::get_associated_token_address;
use spl_token::state::{Account as TokenAccount, AccountState};

const USDC_USDT_POOL: Pubkey = solana_sdk::pubkey!("ARwi1S4DaiTG5DX7S4M4ZsrXqpMD1MrTmbu9ue2tpmEq");
const USDC_MINT: Pubkey = solana_sdk::pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const USDT_MINT: Pubkey = solana_sdk::pubkey!("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB");

#[tokio::test]
async fn test_deposit() {
    let mock_user = Keypair::new();

    let ata_x = get_associated_token_address(&mock_user.pubkey(), &USDC_MINT);
    let ata_y = get_associated_token_address(&mock_user.pubkey(), &USDT_MINT);

    let token_x_account = TokenAccount {
        mint: USDC_MINT,
        owner: mock_user.pubkey(),
        amount: 1_000_000_000_000,
        delegate: COption::None,
        state: AccountState::Initialized,
        is_native: COption::None,
        delegated_amount: 0,
        close_authority: COption::None,
    };

    let mut test = setup_dlmm_vault_program();

    test.prefer_bpf(true);
    test.add_program("dlmm", dlmm::ID, None);

    let mut token_acc_bytes = [0u8; TokenAccount::LEN];
    TokenAccount::pack(token_x_account, &mut token_acc_bytes).unwrap();
    test.add_account(
        ata_x,
        Account {
            lamports: 1_000_000_000,
            data: token_acc_bytes.to_vec(),
            owner: anchor_spl::token::ID,
            executable: false,
            rent_epoch: 0,
        },
    );

    let (mut banks_client, _, _) = test.start().await;

    let ix_data = dlmm_vault::instruction::DlmmDeposit {
        token_x_deposit_amount: 1_000_000,
        token_y_deposit_amount: 0,
    }
    .data();

    let accounts = dlmm_vault::accounts::DlmmDeposit {
        vault_account: USDC_USDT_POOL,
        signer: mock_user.pubkey(),
        vault_owner_token_x: ata_x,
        vault_token_x_account: ata_x,
        vault_owner_token_y: ata_y,
        vault_token_y_account: ata_y,
        token_x_mint: USDC_MINT,
        token_y_mint: USDT_MINT,
        token_x_program: anchor_spl::token::ID,
        token_y_program: anchor_spl::token::ID,
    }
    .to_account_metas(None);

    let instruction = Instruction {
        program_id: dlmm_vault::id(),
        data: ix_data,
        accounts,
    };

    process_and_assert_ok(
        &[
            ComputeBudgetInstruction::set_compute_unit_limit(1_400_000),
            instruction,
        ],
        &mock_user,
        &[&mock_user],
        &mut banks_client,
    )
    .await;
}
