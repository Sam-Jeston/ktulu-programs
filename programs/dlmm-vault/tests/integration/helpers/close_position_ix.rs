use anchor_lang::InstructionData;
use anchor_lang::ToAccountMetas;
use solana_instruction::account_meta::AccountMeta as SAccountMeta;
use solana_message::Instruction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::{signature::Keypair, signer::Signer};

pub fn close_position_ix(
    user: &Keypair,
    vault_account: &Pubkey,
    position: &Pubkey,
    dlmm_program: &Pubkey,
    event_authority: &Pubkey,
) -> Instruction {
    let ix_data = dlmm_vault::instruction::ClosePosition {}.data();

    let accounts = dlmm_vault::accounts::DlmmClosePosition {
        vault_account: vault_account.clone(),
        position: position.clone(),
        dlmm_program: dlmm_program.clone(),
        event_authority: event_authority.clone(),
        signer: user.pubkey(),
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
