use anchor_lang::{InstructionData, ToAccountMetas};
use solana_instruction::account_meta::AccountMeta as SAccountMeta;
use solana_message::Instruction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::{signature::Keypair, signer::Signer};

pub fn deposit_vault_ix(
    user: &Keypair,
    vault_account: &Pubkey,
    vault_owner_token_x: &Pubkey,
    vault_token_x_account: &Pubkey,
    vault_owner_token_y: &Pubkey,
    vault_token_y_account: &Pubkey,
    token_x_mint: &Pubkey,
    token_y_mint: &Pubkey,
    token_x_program: &Pubkey,
    token_y_program: &Pubkey,
    token_x_deposit_amount: u64,
    token_y_deposit_amount: u64,
) -> Instruction {
    let ix_data = dlmm_vault::instruction::DlmmDeposit {
        token_x_deposit_amount,
        token_y_deposit_amount,
    }
    .data();

    let accounts = dlmm_vault::accounts::DlmmDeposit {
        vault_account: vault_account.clone(),
        signer: user.pubkey(),
        vault_owner_token_x: vault_owner_token_x.clone(),
        vault_token_x_account: vault_token_x_account.clone(),
        vault_owner_token_y: vault_owner_token_y.clone(),
        vault_token_y_account: vault_token_y_account.clone(),
        token_x_mint: token_x_mint.clone(),
        token_y_mint: token_y_mint.clone(),
        token_x_program: token_x_program.clone(),
        token_y_program: token_y_program.clone(),
    }
    .to_account_metas(None);

    Instruction {
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
    }
}
