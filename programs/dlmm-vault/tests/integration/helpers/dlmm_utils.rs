use super::dlmm_pda::*;
use anchor_lang::Discriminator;
use anchor_lang::prelude::Pubkey;
use anchor_spl::associated_token::get_associated_token_address;
use anchor_spl::token::spl_token::state::AccountState;
use bytemuck::pod_read_unaligned;
use dlmm_vault::dlmm::accounts::LbPair;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_program_test::ProgramTest;
use solana_sdk::account::Account;

use super::RPC;
use super::utils::add_packable_account;

/// Get bin array index from bin id
pub fn bin_id_to_bin_array_index(bin_id: i32) -> Option<i32> {
    use dlmm_vault::dlmm::constants::MAX_BIN_PER_ARRAY;
    let idx = bin_id.checked_div(MAX_BIN_PER_ARRAY as i32)?;
    let rem = bin_id.checked_rem(MAX_BIN_PER_ARRAY as i32)?;

    if bin_id.is_negative() && rem != 0 {
        idx.checked_sub(1)
    } else {
        Some(idx)
    }
}

pub struct PoolSetupContext {
    pub pool_state: LbPair,
    pub user_token_x: Pubkey,
    pub user_token_y: Pubkey,
}

async fn load_lb_pair_unaligned(acc: &Account) -> anyhow::Result<LbPair> {
    let data = acc.data.as_slice();
    anyhow::ensure!(data.len() >= 8, "account too small for discriminator");

    let (disc, body) = data.split_at(8);
    anyhow::ensure!(
        disc == LbPair::DISCRIMINATOR,
        "wrong discriminator for LbPair"
    );

    let need = core::mem::size_of::<LbPair>();
    anyhow::ensure!(body.len() >= need, "account too small for LbPair body");

    let lbpair = pod_read_unaligned::<LbPair>(&body[..need]);
    Ok(lbpair)
}

pub async fn setup_pool_from_cluster(
    test: &mut ProgramTest,
    pool: Pubkey,
    mock_user: Pubkey,
) -> PoolSetupContext {
    let rpc_client = RpcClient::new(RPC.to_owned());
    let pool_account = rpc_client.get_account(&pool).await.unwrap();

    println!("pool: {:?}", pool);
    let pool_state = load_lb_pair_unaligned(&pool_account).await.unwrap();

    test.add_account(pool, pool_account);

    let (oracle_key, _bump) = derive_oracle_pda(pool);
    let oracle_account = rpc_client.get_account(&oracle_key).await.unwrap();
    test.add_account(oracle_key, oracle_account);

    let active_bin_array_idx = bin_id_to_bin_array_index(pool_state.active_id).unwrap();
    let (active_bin_array_key, _bump) = derive_bin_array_pda(pool, active_bin_array_idx.into());

    let bin_array_account = rpc_client.get_account(&active_bin_array_key).await.unwrap();
    test.add_account(active_bin_array_key, bin_array_account);

    let mint_keys = vec![pool_state.token_x_mint, pool_state.token_y_mint];
    let mints = rpc_client.get_multiple_accounts(&mint_keys).await.unwrap();

    for (key, account) in mint_keys.iter().zip(mints) {
        test.add_account(*key, account.unwrap());
    }

    let reserve_keys = vec![pool_state.reserve_x, pool_state.reserve_y];

    let tokens = rpc_client
        .get_multiple_accounts(&reserve_keys)
        .await
        .unwrap();

    for (key, account) in reserve_keys.into_iter().zip(tokens) {
        test.add_account(key, account.unwrap());
    }

    test.add_account(
        mock_user,
        Account {
            lamports: u32::MAX.into(),
            data: vec![],
            owner: solana_sdk::system_program::ID,
            ..Default::default()
        },
    );

    let token_ata_key = mint_keys
        .iter()
        .map(|key| get_associated_token_address(&mock_user, key))
        .collect::<Vec<_>>();

    for (ata_key, mint_key) in token_ata_key.iter().zip(mint_keys) {
        let state = anchor_spl::token::spl_token::state::Account {
            mint: mint_key,
            owner: mock_user,
            amount: u64::MAX / 2,
            state: AccountState::Initialized,
            ..Default::default()
        };

        add_packable_account(test, state, anchor_spl::token::ID, *ata_key);
    }

    PoolSetupContext {
        pool_state,
        user_token_x: token_ata_key[0],
        user_token_y: token_ata_key[1],
    }
}
