use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount};

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
        token::mint = base_coin,
        token::authority = authority,
    )]
    pub authority_base_coin_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        constraint = base_coin.key() == authority.load()?.base_coin @CustomError::UnauthorizedBaseCoin,
    )]
    pub base_coin: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        mut,
        constraint = mint.key() == authority.load()?.mint @CustomError::IncorrectMint,
    )]
    pub mint: InterfaceAccount<'info, Mint>,
}

pub fn close_account_handler(ctx: Context<CloseAccountCtx>) -> Result<()> {
    require!(ctx.accounts.mint.supply == 0, CustomError::MintIsNotZero);
    require!(
        ctx.accounts.authority_base_coin_token_account.amount == 0,
        CustomError::BaseCoinIsNotZero
    );

    Ok(())
}
