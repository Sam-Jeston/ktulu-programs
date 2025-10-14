use anchor_lang::{system_program, InstructionData, ToAccountMetas};
use dlmm_vault::{FeeCompoundingStrategy, VolatilityStrategy};
use solana_instruction::account_meta::AccountMeta as SAccountMeta;
use solana_message::Instruction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::{signature::Keypair, signer::Signer};
use spl_associated_token_account::get_associated_token_address_with_program_id;

pub fn initialize_vault_ix(
    user: &Keypair,
    operator: &Keypair,
    mint_x: &Pubkey,
    mint_y: &Pubkey,
    dlmm_pool: &Pubkey,
    token_x_program: &Pubkey,
    token_y_program: &Pubkey,
    auto_compound: bool,
    auto_rebalance: bool,
    fee_compounding_strategy: FeeCompoundingStrategy,
    volatility_strategy: VolatilityStrategy,
    bin_width: u16,
    use_harvest_mint: bool,
    harvest_bps: u16,
    harvest_mint: &Pubkey,
    harvest_program: &Pubkey,
    amount_x: u64,
    amount_y: u64,
    user_x_ata: &Pubkey,
    user_y_ata: &Pubkey,
) -> (Instruction, Pubkey, Pubkey, Pubkey, Pubkey) {
    let initialize_ix_data = dlmm_vault::instruction::Initialize {
        auto_compound,
        auto_rebalance,
        volatility_strategy,
        bin_width,
        fee_compounding_strategy,
        operator: operator.pubkey(),
        use_harvest_mint,
        harvest_bps,
        amount_x,
        amount_y,
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

    let (harvest_pda, _bump) =
        Pubkey::find_program_address(&[b"harvest", vault_pda.as_ref()], &dlmm_vault::id());

    let vault_ata_x =
        get_associated_token_address_with_program_id(&vault_pda, &mint_x.clone(), token_x_program);
    let vault_ata_y =
        get_associated_token_address_with_program_id(&vault_pda, &mint_y.clone(), token_y_program);

    let initialize_accounts = dlmm_vault::accounts::Initialize {
        vault_account: vault_pda,
        signer: user.pubkey(),
        token_x_mint: mint_x.clone(),
        token_y_mint: mint_y.clone(),
        token_x_program: token_x_program.clone(),
        token_y_program: token_y_program.clone(),
        harvest_mint: harvest_mint.clone(),
        harvest_program: harvest_program.clone(),
        harvest_account: harvest_pda,
        dlmm_pool: dlmm_pool.to_bytes().into(),
        system_program: system_program::ID,
        token_x_ata: vault_ata_x,
        token_y_ata: vault_ata_y,
        associated_token_program: anchor_spl::associated_token::ID,
        vault_owner_token_x: user_x_ata.clone(),
        vault_owner_token_y: user_y_ata.clone(),
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

    (ix, vault_pda, vault_ata_x, vault_ata_y, harvest_pda)
}
