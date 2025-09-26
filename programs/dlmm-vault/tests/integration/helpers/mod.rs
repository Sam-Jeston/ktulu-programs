use anchor_lang::prelude::*;
use solana_program_test::*;
use solana_sdk::{entrypoint::ProgramResult, pubkey::Pubkey};

pub mod dlmm_pda;
pub mod dlmm_utils;

mod utils;

pub use utils::process_and_assert_ok;
const RPC: &str = "https://api.mainnet-beta.solana.com";

pub fn setup_dlmm_vault_program() -> ProgramTest {
    ProgramTest::new("dlmm_vault", dlmm_vault::ID, processor!(entry))
}

/// This is a wrapper to get the processor macro to work.
fn entry(program_id: &Pubkey, accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    let accounts = Box::leak(Box::new(accounts.to_vec()));
    dlmm_vault::entry(program_id, accounts, instruction_data)
}
