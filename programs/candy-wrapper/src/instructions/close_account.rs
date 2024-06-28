use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::{error::CustomError, state::Authority};

#[derive(Accounts)]
pub struct CloseAccountCtx<'info> {
    #[account(
        mut,
        constraint = payer.key() == authority.load()?.admin,
    )]
    pub payer: Signer<'info>,
    #[account(
        mut,
        close = payer
    )]
    pub authority: AccountLoader<'info, Authority>,
    #[account(
        mut,
        constraint = mint.key() == authority.load()?.mint @CustomError::IncorrectMint,
    )]
    pub mint: InterfaceAccount<'info, Mint>,
}

pub fn close_account_handler(ctx: Context<CloseAccountCtx>) -> Result<()> {
    require!(ctx.accounts.mint.supply == 0, CustomError::MintIsNotZero);

    Ok(())
}
