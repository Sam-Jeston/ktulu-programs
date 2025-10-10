use std::str::FromStr;

use crate::harvest::HarvestEvent;
use crate::{
    ensure_signer_is_owner_or_operator, events::rebalance::RebalanceEvent,
    jupiter::program::Jupiter, token_amount, DlmmVaultAccount,
};
use crate::{mul_div_floor_u64, VaultErrorCode};
use anchor_lang::prelude::*;
use anchor_lang::solana_program::{instruction::Instruction, program::invoke_signed};
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

pub fn jupiter_program_id() -> Pubkey {
    Pubkey::from_str("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4").unwrap()
}

#[derive(Accounts)]
pub struct Rebalance<'info> {
    #[account(
        mut,
        seeds = [b"dlmm_vault".as_ref(), vault_account.owner.as_ref(), vault_account.dlmm_pool_id.as_ref()],
        bump
    )]
    pub vault_account: Account<'info, DlmmVaultAccount>,
    #[account(mut)]
    pub signer: Signer<'info>,

    pub input_mint: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = input_mint,
        associated_token::authority = vault_account,
        associated_token::token_program = input_token_program
    )]
    pub vault_input_token_account: InterfaceAccount<'info, TokenAccount>,
    pub input_token_program: Interface<'info, TokenInterface>,

    pub output_mint: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = output_mint,
        associated_token::authority = vault_account,
        associated_token::token_program = output_token_program
    )]
    pub vault_output_token_account: InterfaceAccount<'info, TokenAccount>,
    pub output_token_program: Interface<'info, TokenInterface>,

    #[account(
        mut,
        associated_token::mint = output_mint,
        associated_token::authority = vault_account.operator,
        associated_token::token_program = output_token_program
    )]
    // Fees must be collected in the ATA for the operator to ensure
    // we can confirm this is the destination on the Jup swap and that
    // the balance change is not malicious
    pub operator_fee_account: InterfaceAccount<'info, TokenAccount>,

    pub jupiter_program: Program<'info, Jupiter>,
}

pub fn handle_rebalance<'a, 'b, 'c, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, Rebalance<'info>>,
    data: Vec<u8>,
) -> Result<()> {
    ensure_signer_is_owner_or_operator(&ctx.accounts.signer.key, &ctx.accounts.vault_account)?;
    require_keys_eq!(*ctx.accounts.jupiter_program.key, jupiter_program_id());

    let rebalance_is_enabled = ctx.accounts.vault_account.auto_rebalance;
    let harvest_is_enabled = ctx.accounts.vault_account.use_harvest_mint;
    require!(
        rebalance_is_enabled || harvest_is_enabled,
        VaultErrorCode::AutoRebalanceOrHarvestNotEnabled
    );

    let initial_in_balance = ctx.accounts.vault_input_token_account.amount;
    let initial_out_balance = ctx.accounts.vault_output_token_account.amount;
    let initial_operator_fee_balance = ctx.accounts.operator_fee_account.amount;

    let accounts: Vec<AccountMeta> = ctx
        .remaining_accounts
        .iter()
        .map(|acc| {
            let is_signer = acc.key == &ctx.accounts.vault_account.key();
            AccountMeta {
                pubkey: *acc.key,
                is_signer,
                is_writable: acc.is_writable,
            }
        })
        .collect();

    // Regardless of the exact routing method, at the latest we should have seen our source and destination vault accounts by,
    // at the very latest, index 6.
    // Validate ctx.accounts.vault_input_token_account.amount is in accounts by at the latest accounts[6]
    let mut vault_input_included = false;
    let mut vault_output_included = false;
    for (i, acc) in accounts.iter().enumerate() {
        if i > 6 {
            break;
        }
        if acc.pubkey == ctx.accounts.vault_input_token_account.key() {
            vault_input_included = true;
        }
        if acc.pubkey == ctx.accounts.vault_output_token_account.key() {
            vault_output_included = true;
        }
    }

    // Platform fee account is between instruction 6 and 10
    let mut operator_fee_account_included = false;
    let mut i = 6;
    while i <= 10 {
        if accounts[i].pubkey == ctx.accounts.operator_fee_account.key() {
            operator_fee_account_included = true;
        }

        i += 1;
    }

    // NOTE: we could get more precise and match the descriminator of the data to know exactly which accounts are which,
    // but this is pretty safe, as it prevents platform fee account being included, which is where an attacker might try
    // to drain to pass the balance validation post-swap
    require!(vault_input_included, VaultErrorCode::InvalidTokenAccount);
    require!(vault_output_included, VaultErrorCode::InvalidTokenAccount);
    require!(
        operator_fee_account_included,
        VaultErrorCode::InvalidTokenAccount
    );

    let accounts_infos: Vec<AccountInfo> = ctx
        .remaining_accounts
        .iter()
        .map(|acc| AccountInfo { ..acc.clone() })
        .collect();

    let signer_seeds: &[&[&[u8]]] = &[&[
        b"dlmm_vault",
        ctx.accounts.vault_account.owner.as_ref(),
        ctx.accounts.vault_account.dlmm_pool_id.as_ref(),
        &[ctx.bumps.vault_account],
    ]];

    invoke_signed(
        &Instruction {
            program_id: ctx.accounts.jupiter_program.key(),
            accounts,
            data,
        },
        &accounts_infos,
        signer_seeds,
    )?;

    // Re-read the token accounts to determine the final balances
    let final_in_info = ctx.accounts.vault_input_token_account.to_account_info();
    let final_out_info = ctx.accounts.vault_output_token_account.to_account_info();
    let final_in_balance = token_amount(&final_in_info)?;
    let final_out_balance = token_amount(&final_out_info)?;
    let final_operator_fee_balance =
        token_amount(&ctx.accounts.operator_fee_account.to_account_info())?;

    // Last safety step. Final in should be less than initial in
    require!(
        final_in_balance < initial_in_balance,
        VaultErrorCode::InvalidSwapAmount
    );
    require!(
        final_out_balance > initial_out_balance,
        VaultErrorCode::InvalidSwapAmount
    );

    // Max operator fee should be 10bps of the total out.
    let out_diff = final_out_balance - initial_out_balance;
    let operator_fee_diff = final_operator_fee_balance - initial_operator_fee_balance;
    let max_operator_fee = mul_div_floor_u64(out_diff, 10, 10_000);
    require!(
        operator_fee_diff <= max_operator_fee,
        VaultErrorCode::InvalidOperatorFee
    );

    emit!(RebalanceEvent {
        vault_account: ctx.accounts.vault_account.key(),
        in_mint: ctx.accounts.input_mint.key(),
        out_mint: ctx.accounts.output_mint.key(),
        initial_in_balance: initial_in_balance,
        initial_out_balance: initial_out_balance,
        final_in_balance: final_in_balance,
        final_out_balance: final_out_balance,
        signer: ctx.accounts.signer.key(),
    });

    Ok(())
}

#[derive(Accounts)]
pub struct Harvest<'info> {
    #[account(
        mut,
        seeds = [b"dlmm_vault".as_ref(), vault_account.owner.as_ref(), vault_account.dlmm_pool_id.as_ref()],
        bump
    )]
    pub vault_account: Account<'info, DlmmVaultAccount>,
    #[account(mut)]
    pub signer: Signer<'info>,

    pub input_mint: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = input_mint,
        associated_token::authority = vault_account,
        associated_token::token_program = input_token_program
    )]
    pub vault_input_token_account: InterfaceAccount<'info, TokenAccount>,
    pub input_token_program: Interface<'info, TokenInterface>,

    pub output_mint: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = output_mint,
        associated_token::authority = vault_account,
        associated_token::token_program = output_token_program,
    )]
    pub vault_output_token_account: InterfaceAccount<'info, TokenAccount>,
    pub output_token_program: Interface<'info, TokenInterface>,

    #[account(
        mut,
        associated_token::mint = output_mint,
        associated_token::authority = vault_account.operator,
        associated_token::token_program = output_token_program
    )]
    // Fees must be collected in the ATA for the operator to ensure
    // we can confirm this is the destination on the Jup swap and that
    // the balance change is not malicious
    pub operator_fee_account: InterfaceAccount<'info, TokenAccount>,

    pub jupiter_program: Program<'info, Jupiter>,
}

pub fn handle_harvest<'a, 'b, 'c, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, Harvest<'info>>,
    data: Vec<u8>,
) -> Result<()> {
    ensure_signer_is_owner_or_operator(&ctx.accounts.signer.key, &ctx.accounts.vault_account)?;
    require_keys_eq!(*ctx.accounts.jupiter_program.key, jupiter_program_id());

    let rebalance_is_enabled = ctx.accounts.vault_account.auto_rebalance;
    let harvest_is_enabled = ctx.accounts.vault_account.use_harvest_mint;
    require!(
        rebalance_is_enabled || harvest_is_enabled,
        VaultErrorCode::AutoRebalanceOrHarvestNotEnabled
    );

    let initial_in_balance = ctx.accounts.vault_input_token_account.amount;
    let initial_out_balance = ctx.accounts.vault_output_token_account.amount;
    let initial_operator_fee_balance = ctx.accounts.operator_fee_account.amount;

    let accounts: Vec<AccountMeta> = ctx
        .remaining_accounts
        .iter()
        .map(|acc| {
            let is_signer = acc.key == &ctx.accounts.vault_account.key();
            AccountMeta {
                pubkey: *acc.key,
                is_signer,
                is_writable: acc.is_writable,
            }
        })
        .collect();

    // Regardless of the exact routing method, at the latest we should have seen our source and destination vault accounts by,
    // at the very latest, index 6.
    // Validate ctx.accounts.vault_input_token_account.amount is in accounts by at the latest accounts[6]
    let mut vault_input_included = false;
    let mut vault_output_included = false;
    for (i, acc) in accounts.iter().enumerate() {
        if i > 6 {
            break;
        }
        if acc.pubkey == ctx.accounts.vault_input_token_account.key() {
            vault_input_included = true;
        }
        if acc.pubkey == ctx.accounts.vault_output_token_account.key() {
            vault_output_included = true;
        }
    }

    // Platform fee account is between instruction 6 and 10
    let mut operator_fee_account_included = false;
    let mut i = 6;
    while i <= 10 {
        if accounts[i].pubkey == ctx.accounts.operator_fee_account.key() {
            operator_fee_account_included = true;
        }

        i += 1;
    }

    // NOTE: we could get more precise and match the descriminator of the data to know exactly which accounts are which,
    // but this is pretty safe, as it prevents platform fee account being included, which is where an attacker might try
    // to drain to pass the balance validation post-swap
    require!(vault_input_included, VaultErrorCode::InvalidTokenAccount);
    require!(vault_output_included, VaultErrorCode::InvalidTokenAccount);
    require!(
        operator_fee_account_included,
        VaultErrorCode::InvalidTokenAccount
    );

    let accounts_infos: Vec<AccountInfo> = ctx
        .remaining_accounts
        .iter()
        .map(|acc| AccountInfo { ..acc.clone() })
        .collect();

    let signer_seeds: &[&[&[u8]]] = &[&[
        b"dlmm_vault",
        ctx.accounts.vault_account.owner.as_ref(),
        ctx.accounts.vault_account.dlmm_pool_id.as_ref(),
        &[ctx.bumps.vault_account],
    ]];

    invoke_signed(
        &Instruction {
            program_id: ctx.accounts.jupiter_program.key(),
            accounts,
            data,
        },
        &accounts_infos,
        signer_seeds,
    )?;

    // Re-read the token accounts to determine the final balances
    let final_in_info = ctx.accounts.vault_input_token_account.to_account_info();
    let final_out_info = ctx.accounts.vault_output_token_account.to_account_info();
    let final_in_balance = token_amount(&final_in_info)?;
    let final_out_balance = token_amount(&final_out_info)?;
    let final_operator_fee_balance =
        token_amount(&ctx.accounts.operator_fee_account.to_account_info())?;

    // Last safety step. Final in should be less than initial in
    require!(
        final_in_balance < initial_in_balance,
        VaultErrorCode::InvalidSwapAmount
    );
    require!(
        final_out_balance > initial_out_balance,
        VaultErrorCode::InvalidSwapAmount
    );

    // Ensure the harvest fee account has not consumed more than 1% of the output
    let out_diff = final_out_balance - initial_out_balance;
    let operator_fee_diff = final_operator_fee_balance - initial_operator_fee_balance;
    let max_operator_fee = mul_div_floor_u64(out_diff, 110, 10_000);
    require!(
        operator_fee_diff <= max_operator_fee,
        VaultErrorCode::InvalidOperatorFee
    );

    emit!(HarvestEvent {
        vault_account: ctx.accounts.vault_account.key(),
        in_mint: ctx.accounts.input_mint.key(),
        out_mint: ctx.accounts.output_mint.key(),
        initial_in_balance: initial_in_balance,
        initial_out_balance: initial_out_balance,
        final_in_balance: final_in_balance,
        final_out_balance: final_out_balance,
        signer: ctx.accounts.signer.key(),
    });

    Ok(())
}
