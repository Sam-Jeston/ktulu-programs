use anchor_lang::{system_program, InstructionData, ToAccountMetas};
use solana_instruction::account_meta::AccountMeta as SAccountMeta;
use solana_message::Instruction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::{signature::Keypair, signer::Signer};
use spl_associated_token_account::get_associated_token_address_with_program_id;

pub fn initialize_vault_ix(
    user: &Keypair,
    mint_x: &Pubkey,
    mint_y: &Pubkey,
    dlmm_pool: &Pubkey,
) -> (Instruction, Pubkey, Pubkey, Pubkey) {
    let initialize_ix_data = dlmm_vault::instruction::Initialize {
        token_x_mint: mint_x.clone(),
        token_y_mint: mint_y.clone(),
        lower_price_range_bps: 0,
        upper_price_range_bps: 0,
        operator: user.pubkey(),
    }
    .data();

    let (vault_pda, _bump) = Pubkey::find_program_address(
        &[
            b"dlmm_vault",
            user.pubkey().as_ref(),
            dlmm_pool.as_ref(), // dlmm_pool.key
        ],
        &dlmm_vault::id(),
    );

    let vault_ata_x = get_associated_token_address_with_program_id(
        &vault_pda,
        &mint_x.clone(),
        &anchor_spl::token::ID,
    );
    let vault_ata_y = get_associated_token_address_with_program_id(
        &vault_pda,
        &mint_y.clone(),
        &anchor_spl::token::ID,
    );

    let initialize_accounts = dlmm_vault::accounts::Initialize {
        vault_account: vault_pda,
        signer: user.pubkey(),
        token_x_mint: mint_x.clone(),
        token_y_mint: mint_y.clone(),
        token_x_program: anchor_spl::token::ID,
        token_y_program: anchor_spl::token::ID,
        dlmm_pool: dlmm_pool.to_bytes().into(),
        system_program: system_program::ID,
        token_x_ata: vault_ata_x,
        token_y_ata: vault_ata_y,
        associated_token_program: anchor_spl::associated_token::ID,
    }
    .to_account_metas(None);

    let ix = Instruction {
        program_id: dlmm_vault::id().to_bytes().into(),
        data: initialize_ix_data,
        accounts: initialize_accounts
            .iter()
            .map(|a| SAccountMeta {
                pubkey: a.pubkey.to_bytes().into(),
                is_signer: a.is_signer,
                is_writable: a.is_writable,
            })
            .collect(),
    };

    (ix, vault_pda, vault_ata_x, vault_ata_y)
}
