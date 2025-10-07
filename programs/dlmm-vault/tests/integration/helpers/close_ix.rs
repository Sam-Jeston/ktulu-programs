use anchor_lang::{InstructionData, ToAccountMetas};
use solana_instruction::account_meta::AccountMeta as SAccountMeta;
use solana_message::Instruction;
use solana_sdk::pubkey::Pubkey;

pub fn close_ix(
    vault_account: &Pubkey,
    owner: &Pubkey,
    vault_token_x: &Pubkey,
    vault_token_y: &Pubkey,
    token_x_mint: &Pubkey,
    token_y_mint: &Pubkey,
    token_x_program: &Pubkey,
    token_y_program: &Pubkey,
    vault_harvest_token: &Pubkey,
    harvest_mint: &Pubkey,
    harvest_program: &Pubkey,
) -> Instruction {
    let ix_data = dlmm_vault::instruction::CloseVault {}.data();

    let accounts = dlmm_vault::accounts::CloseVault {
        vault_account: vault_account.clone(),
        owner: owner.clone(),
        vault_token_x: vault_token_x.clone(),
        vault_token_y: vault_token_y.clone(),
        token_x_mint: token_x_mint.clone(),
        token_y_mint: token_y_mint.clone(),
        token_x_program: token_x_program.clone(),
        token_y_program: token_y_program.clone(),
        vault_harvest_token: vault_harvest_token.clone(),
        harvest_mint: harvest_mint.clone(),
        harvest_program: harvest_program.clone(),
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
