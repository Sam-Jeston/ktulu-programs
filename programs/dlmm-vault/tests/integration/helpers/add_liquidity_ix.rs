use anchor_lang::{InstructionData, ToAccountMetas};
use dlmm_vault::dlmm::types::BinLiquidityDistribution;
use solana_instruction::account_meta::AccountMeta as SAccountMeta;
use solana_message::Instruction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::{signature::Keypair, signer::Signer};

// Account reference
// pub vault_owner: UncheckedAccount<'info>,
// pub vault_account: Account<'info, DlmmVaultAccount>,
// #[account(mut)]
// /// CHECK: The pool account
// pub lb_pair: UncheckedAccount<'info>,

// /// CHECK: Bin array extension account of the pool
// pub bin_array_bitmap_extension: Option<UncheckedAccount<'info>>,

// #[account(mut)]
// /// CHECK: Reserve account of token X
// pub reserve_x: UncheckedAccount<'info>,
// #[account(mut)]
// /// CHECK: Reserve account of token Y
// pub reserve_y: UncheckedAccount<'info>,

// #[account(mut)]
// /// CHECK: User token X account
// pub user_token_x: UncheckedAccount<'info>,
// #[account(mut)]
// /// CHECK: User token Y account
// pub user_token_y: UncheckedAccount<'info>,

// /// CHECK: Mint account of token X
// pub token_x_mint: UncheckedAccount<'info>,
// /// CHECK: Mint account of token Y
// pub token_y_mint: UncheckedAccount<'info>,

// #[account(mut)]
// /// CHECK: Oracle account of the pool
// pub oracle: UncheckedAccount<'info>,

// #[account(mut)]
// /// CHECK: The position account to be created
// pub position: UncheckedAccount<'info>,

// /// CHECK: User who's executing the create position. Either the user or vault operator on rebalance
// pub sender: Signer<'info>,

// #[account(address = dlmm::ID)]
// /// CHECK: DLMM program
// pub dlmm_program: UncheckedAccount<'info>,

// /// CHECK: DLMM program event authority for event CPI
// pub event_authority: UncheckedAccount<'info>,

// /// CHECK: Token program of mint X
// pub token_x_program: UncheckedAccount<'info>,
// /// CHECK: Token program of mint Y
// pub token_y_program: UncheckedAccount<'info>,
// /// CHECK: Bin array lower account
// pub bin_array_lower: UncheckedAccount<'info>,
// /// CHECK: Bin array upper account
// pub bin_array_upper: UncheckedAccount<'info>,

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
    oracle: &Pubkey,
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
        oracle: oracle.clone(),
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
