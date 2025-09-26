use crate::dlmm::{self};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct DlmmCreatePosition<'info> {
    #[account(mut)]
    /// CHECK: The pool account
    pub lb_pair: UncheckedAccount<'info>,

    /// CHECK: Bin array extension account of the pool
    pub bin_array_bitmap_extension: Option<UncheckedAccount<'info>>,

    #[account(mut)]
    /// CHECK: Reserve account of token X
    pub reserve_x: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: Reserve account of token Y
    pub reserve_y: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: User token X account
    pub user_token_x: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: User token Y account
    pub user_token_y: UncheckedAccount<'info>,

    /// CHECK: Mint account of token X
    pub token_x_mint: UncheckedAccount<'info>,
    /// CHECK: Mint account of token Y
    pub token_y_mint: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: Oracle account of the pool
    pub oracle: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: The position account to be created
    pub position: UncheckedAccount<'info>,

    #[account(address = dlmm::ID)]
    /// CHECK: DLMM program
    pub dlmm_program: UncheckedAccount<'info>,

    /// CHECK: DLMM program event authority for event CPI
    pub event_authority: UncheckedAccount<'info>,

    /// CHECK: Token program of mint X
    pub token_x_program: UncheckedAccount<'info>,
    /// CHECK: Token program of mint Y
    pub token_y_program: UncheckedAccount<'info>,

    pub owner: Signer<'info>,
    pub payer: Signer<'info>,
    pub rent: UncheckedAccount<'info>,
    pub system_program: UncheckedAccount<'info>,
}

pub fn handle_dlmm_create_position<'a, 'b, 'c, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, DlmmCreatePosition<'info>>,
    lower_bin_id: i32,
    width: i32,
) -> Result<()> {
    let accounts = dlmm::cpi::accounts::InitializePosition {
        owner: ctx.accounts.owner.to_account_info(),
        payer: ctx.accounts.payer.to_account_info(),
        rent: ctx.accounts.rent.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
        lb_pair: ctx.accounts.lb_pair.to_account_info(),
        position: ctx.accounts.position.to_account_info(),
        event_authority: ctx.accounts.event_authority.to_account_info(),
        program: ctx.accounts.dlmm_program.to_account_info(),
    };

    let cpi_context = CpiContext::new(ctx.accounts.dlmm_program.to_account_info(), accounts)
        .with_remaining_accounts(ctx.remaining_accounts.to_vec());

    dlmm::cpi::initialize_position(cpi_context, lower_bin_id, width)
}
