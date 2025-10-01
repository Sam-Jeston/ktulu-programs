use crate::helpers;
use anchor_lang::{solana_program::pubkey::Pubkey, InstructionData, ToAccountMetas};
use dlmm_vault::dlmm;
use helpers::dlmm_utils::*;
use helpers::{process_and_assert_ok, setup_dlmm_vault_program};
use solana_program_test::*;
use solana_sdk::instruction::AccountMeta;
use solana_sdk::{
    compute_budget::ComputeBudgetInstruction, instruction::Instruction, signature::Keypair,
    signer::Signer,
};

const USDC_USDT_POOL: Pubkey = solana_sdk::pubkey!("ARwi1S4DaiTG5DX7S4M4ZsrXqpMD1MrTmbu9ue2tpmEq");

#[tokio::test]
async fn test_dlmm_swap() {
    let mock_user = Keypair::new();

    let mut test = setup_dlmm_vault_program();

    test.prefer_bpf(true);
    test.add_program("dlmm", dlmm::ID, None);

    let PoolSetupContext {
        pool_state,
        user_token_x,
        user_token_y,
    } = setup_pool_from_cluster(&mut test, USDC_USDT_POOL, mock_user.pubkey()).await;
}
