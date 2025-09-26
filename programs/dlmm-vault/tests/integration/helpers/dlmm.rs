use std::collections::HashSet;

use anchor_lang::Discriminator;
use bytemuck::pod_read_unaligned;
use dlmm_vault::dlmm::accounts::LbPair;
use litesvm::LiteSVM;
use solana_account::Account;
use solana_sdk::pubkey::Pubkey;

use crate::helpers::{
    account::load_account,
    dlmm_pda::{derive_bin_array_pda, derive_oracle_pda},
};

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

pub fn load_dlmm_accounts(svm: &mut LiteSVM, pool: &Pubkey) -> anyhow::Result<LbPair> {
    let pool_account = load_account(svm, pool);
    let pool_state = load_lb_pair_unaligned(&pool_account).unwrap();

    let (oracle_key, _bump) = derive_oracle_pda(pool.clone());
    load_account(svm, &oracle_key);

    // Load 20 unique bin array keys under and above the active bin
    let mut bin_array_keys: HashSet<Pubkey> = HashSet::new();
    for i in -20..=20 {
        let bin_id = pool_state.active_id + i;
        let active_bin_array_idx = bin_id_to_bin_array_index(bin_id).unwrap();
        let (bin_array_key, _bump) =
            derive_bin_array_pda(pool.clone(), active_bin_array_idx.into());
        bin_array_keys.insert(bin_array_key);
    }

    for bin_array_key in bin_array_keys.iter() {
        load_account(svm, bin_array_key);
    }

    let mint_keys = vec![pool_state.token_x_mint, pool_state.token_y_mint];
    for key in mint_keys.iter() {
        load_account(svm, key);
    }

    let reserve_keys = vec![pool_state.reserve_x, pool_state.reserve_y];
    for key in reserve_keys.into_iter() {
        load_account(svm, &key);
    }

    Ok(pool_state)
}
