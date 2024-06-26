use anchor_lang::prelude::*;
use anchor_spl::{
    token_2022::{
        spl_token_2022::extension::transfer_fee::instruction::set_transfer_fee, Token2022,
    },
    token_interface::{Mint, TokenInterface},
};
use solana_program::program::invoke;

use crate::{error::CustomError, state::Authority};

#[derive(Accounts)]
pub struct TransferFeeCtx<'info> {
    #[account(
        mut,
        constraint = payer.key() == authority.load()?.admin,
    )]
    pub payer: Signer<'info>,
    pub authority: AccountLoader<'info, Authority>,
    #[account(
        mut,
        constraint = mint.key() == authority.load()?.mint @CustomError::IncorrectMint,
    )]
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(
        address = Token2022::id()
    )]
    pub token_program_2022: Interface<'info, TokenInterface>,
}

pub fn change_transfer_fee_handler(
    ctx: Context<TransferFeeCtx>,
    fee_basis_pts: u16,
    max_fee: u64,
) -> Result<()> {
    let ix = set_transfer_fee(
        ctx.accounts.token_program_2022.key,
        &ctx.accounts.mint.key(),
        &ctx.accounts.payer.key(),
        &[],
        fee_basis_pts,
        max_fee,
    )?;
    invoke(
        &ix,
        &[
            ctx.accounts.mint.to_account_info(),
            ctx.accounts.payer.to_account_info(),
        ],
    )?;
    Ok(())
}
