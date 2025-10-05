use std::str::FromStr;

use crate::{
    ensure_signer_is_owner_or_operator, events::rebalance::RebalanceEvent,
    jupiter::program::Jupiter, token_amount, DlmmVaultAccount,
};
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
    #[account(mut)]
    pub vault_input_token_account: InterfaceAccount<'info, TokenAccount>,
    pub input_token_program: Interface<'info, TokenInterface>,

    pub output_mint: InterfaceAccount<'info, Mint>,
    #[account(mut)]
    pub vault_output_token_account: InterfaceAccount<'info, TokenAccount>,
    pub output_token_program: Interface<'info, TokenInterface>,

    pub jupiter_program: Program<'info, Jupiter>,
}

pub fn handle_rebalance<'a, 'b, 'c, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, Rebalance<'info>>,
    data: Vec<u8>,
) -> Result<()> {
    // TODO: Add "last rebalanced" guard onto VaultAccount to give users guarantees
    // that the operator cannot abuse the rebalance instruction

    ensure_signer_is_owner_or_operator(&ctx.accounts.signer.key, &ctx.accounts.vault_account)?;

    require_keys_eq!(*ctx.accounts.jupiter_program.key, jupiter_program_id());

    let initial_in_balance = ctx.accounts.vault_input_token_account.amount;
    let initial_out_balance = ctx.accounts.vault_output_token_account.amount;

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
