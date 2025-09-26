use anchor_lang::{InstructionData, ToAccountMetas};
use dlmm_vault::dlmm::types::BinLiquidityDistribution;
use solana_instruction::account_meta::AccountMeta as SAccountMeta;
use solana_message::Instruction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::{signature::Keypair, signer::Signer};

pub fn add_liquidity_ix(
    user: &Keypair,
    vault_account: &Pubkey,
    lb_pair: &Pubkey,
    bin_array_bitmap_extension: &Option<Pubkey>,
    reserve_x: &Pubkey,
    reserve_y: &Pubkey,
    user_token_x: &Pubkey,
    user_token_y: &Pubkey,
    token_x_mint: &Pubkey,
    token_y_mint: &Pubkey,
    position: &Pubkey,
    token_x_program: &Pubkey,
    token_y_program: &Pubkey,
    event_authority: &Pubkey,
    dlmm_program: &Pubkey,
    bin_array_lower: &Pubkey,
    bin_array_upper: &Pubkey,
    amount_x: u64,
    amount_y: u64,
    bin_liquidity_dist: Vec<BinLiquidityDistribution>,
) -> Instruction {
    let ix_data = dlmm_vault::instruction::AddLiquidity {
        amount_x,
        amount_y,
        bin_liquidity_dist,
    }
    .data();

    let accounts = dlmm_vault::accounts::DlmmAddLiquidity {
        signer: user.pubkey(),
        vault_account: vault_account.clone(),
        lb_pair: lb_pair.clone(),
        position: position.clone(),
        dlmm_program: dlmm_program.clone(),
        event_authority: event_authority.clone(),
        token_x_program: token_x_program.clone(),
        token_y_program: token_y_program.clone(),
        bin_array_lower: bin_array_lower.clone(),
        bin_array_upper: bin_array_upper.clone(),
        reserve_x: reserve_x.clone(),
        reserve_y: reserve_y.clone(),
        vault_token_x: user_token_x.clone(),
        vault_token_y: user_token_y.clone(),
        token_x_mint: token_x_mint.clone(),
        token_y_mint: token_y_mint.clone(),
        bin_array_bitmap_extension: bin_array_bitmap_extension.clone(),
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
