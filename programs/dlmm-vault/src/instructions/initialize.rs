use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;

use crate::{
    events::initialize::InitializeEvent, DlmmVaultAccount, FeeCompoundingStrategy, VaultErrorCode,
    VolatilityStrategy,
};
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = signer,
        space = 8 + DlmmVaultAccount::INIT_SPACE,
        seeds = [b"dlmm_vault".as_ref(), signer.key().as_ref(), dlmm_pool.key.as_ref()],
        bump
    )]
    pub vault_account: Account<'info, DlmmVaultAccount>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub dlmm_pool: UncheckedAccount<'info>,

    pub token_x_mint: InterfaceAccount<'info, Mint>,
    pub token_x_program: Interface<'info, TokenInterface>,
    #[account(
        init,
        payer = signer,
        associated_token::mint = token_x_mint,
        associated_token::authority = vault_account,
        associated_token::token_program = token_x_program
    )]
    pub token_x_ata: InterfaceAccount<'info, TokenAccount>,

    pub token_y_mint: InterfaceAccount<'info, Mint>,
    pub token_y_program: Interface<'info, TokenInterface>,
    #[account(
        init,
        payer = signer,
        associated_token::mint = token_y_mint,
        associated_token::authority = vault_account,
        associated_token::token_program = token_y_program
    )]
    pub token_y_ata: InterfaceAccount<'info, TokenAccount>,

    pub harvest_mint: InterfaceAccount<'info, Mint>,
    pub harvest_program: Interface<'info, TokenInterface>,
    #[account(
        init,
        payer = signer,
        token::mint = harvest_mint,
        token::authority = vault_account,
        token::token_program = harvest_program,
        seeds = [b"harvest".as_ref(), vault_account.key().as_ref()],
        bump,
    )]
    pub harvest_account: InterfaceAccount<'info, TokenAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn handle_initialize<'a, 'b, 'c, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, Initialize<'info>>,
    auto_compound: bool,
    auto_rebalance: bool,
    fee_compounding_strategy: FeeCompoundingStrategy,
    volatility_strategy: VolatilityStrategy,
    bin_width: u16,
    operator: Pubkey,
    use_harvest_mint: bool,
    harvest_bps: u16,
) -> Result<()> {
    if harvest_bps > 10_000 {
        return Err(error!(VaultErrorCode::InvalidHarvestBps));
    }

    ctx.accounts.vault_account.dlmm_pool_id = ctx.accounts.dlmm_pool.key();
    ctx.accounts.vault_account.token_x_mint = ctx.accounts.token_x_mint.key();
    ctx.accounts.vault_account.token_y_mint = ctx.accounts.token_y_mint.key();
    ctx.accounts.vault_account.harvest_mint = ctx.accounts.harvest_mint.key();
    ctx.accounts.vault_account.volatility_strategy = volatility_strategy.clone();
    ctx.accounts.vault_account.bin_width = bin_width;
    ctx.accounts.vault_account.owner = ctx.accounts.signer.key();
    ctx.accounts.vault_account.operator = operator;
    ctx.accounts.vault_account.in_position = false;
    ctx.accounts.vault_account.position_id = Pubkey::default();
    ctx.accounts.vault_account.use_harvest_mint = use_harvest_mint;
    ctx.accounts.vault_account.harvest_bps = harvest_bps;
    ctx.accounts.vault_account.virtual_token_x_harvest = 0;
    ctx.accounts.vault_account.virtual_token_y_harvest = 0;

    emit!(InitializeEvent {
        vault_account: ctx.accounts.vault_account.key(),
        owner: ctx.accounts.signer.key(),
        token_x_mint: ctx.accounts.token_x_mint.key(),
        token_y_mint: ctx.accounts.token_y_mint.key(),
        dlmm_pool: ctx.accounts.dlmm_pool.key(),
        auto_compound: auto_compound,
        auto_rebalance: auto_rebalance,
        fee_compounding_strategy: fee_compounding_strategy,
        volatility_strategy: volatility_strategy,
        bin_width: bin_width,
        operator: operator,
        position_id: Pubkey::default(),
    });

    Ok(())
}
