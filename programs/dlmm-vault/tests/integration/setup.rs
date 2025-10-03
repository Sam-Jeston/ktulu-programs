use std::collections::HashSet;

use anchor_lang::prelude::Pubkey;
use anchor_lang::Discriminator;
use bytemuck::pod_read_unaligned;
use dlmm_vault::dlmm::accounts::LbPair;
use solana_client::rpc_client::RpcClient;
use solana_sdk::account::Account;

use crate::helpers::dlmm_pda::{derive_bin_array_pda, derive_oracle_pda};

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

fn load_lb_pair_unaligned(acc: &Account) -> anyhow::Result<LbPair> {
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

fn print_account_dump(pubkey: &Pubkey) {
    println!(
        "solana account {} --output json > fixtures/{}.json",
        pubkey, pubkey
    );
}

const RPC: &str = "https://api.mainnet-beta.solana.com";
const PUMP_USDC_POOL: Pubkey = solana_sdk::pubkey!("9SMp4yLKGtW9TnLimfVPkDARsyNSfJw43WMke4r7KoZj");

// The stdout from this test is added to ./refresh-fixtures.sh to allow the tests
// to test against the pool provided
#[test]
pub fn setup_pool_from_cluster() {
    println!(
        "# Account dumps for PUMP-USDC DLMM pool 9SMp4yLKGtW9TnLimfVPkDARsyNSfJw43WMke4r7KoZj"
    );
    let rpc_client = RpcClient::new(RPC.to_owned());
    let pool_account = rpc_client.get_account(&PUMP_USDC_POOL).unwrap();
    print_account_dump(&PUMP_USDC_POOL);

    let pool_state = load_lb_pair_unaligned(&pool_account).unwrap();

    let (oracle_key, _bump) = derive_oracle_pda(PUMP_USDC_POOL);
    print_account_dump(&oracle_key);

    println!("# Active bin ID: {}", pool_state.active_id);

    // Load 20 unique bin array keys under and above the active bin
    let mut bin_array_keys: HashSet<Pubkey> = HashSet::new();
    for i in -20..=20 {
        let bin_id = pool_state.active_id + i;
        let active_bin_array_idx = bin_id_to_bin_array_index(bin_id).unwrap();
        let (bin_array_key, _bump) =
            derive_bin_array_pda(PUMP_USDC_POOL, active_bin_array_idx.into());
        bin_array_keys.insert(bin_array_key);
    }

    for bin_array_key in bin_array_keys.iter() {
        print_account_dump(bin_array_key);
    }

    let mint_keys = vec![pool_state.token_x_mint, pool_state.token_y_mint];

    for key in mint_keys.iter() {
        print_account_dump(key);
    }

    let reserve_keys = vec![pool_state.reserve_x, pool_state.reserve_y];
    for key in reserve_keys.into_iter() {
        print_account_dump(&key);
    }
}
