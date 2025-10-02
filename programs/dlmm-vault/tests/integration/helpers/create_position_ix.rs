use anchor_lang::{InstructionData, ToAccountMetas};
use solana_instruction::account_meta::AccountMeta as SAccountMeta;
use solana_message::Instruction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::{signature::Keypair, signer::Signer};

pub fn create_position_ix(
    user: &Keypair,
    vault_account: &Pubkey,
    lb_pair: &Pubkey,
    position: &Pubkey,
    dlmm_program: &Pubkey,
    event_authority: &Pubkey,
    rent: &Pubkey,
    system_program: &Pubkey,
    lower_bin_id: i32,
    width: i32,
) -> Instruction {
    let ix_data = dlmm_vault::instruction::CreatePosition {
        lower_bin_id,
        width,
    }
    .data();

    let accounts = dlmm_vault::accounts::DlmmCreatePosition {
        vault_account: vault_account.clone(),
        lb_pair: lb_pair.clone(),
        position: position.clone(),
        dlmm_program: dlmm_program.clone(),
        event_authority: event_authority.clone(),
        rent: rent.clone(),
        system_program: system_program.clone(),
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
