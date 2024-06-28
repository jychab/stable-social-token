use anchor_lang::prelude::*;

use crate::{error::CustomError, state::Authority};

#[derive(Accounts)]
pub struct FeeCollectorCtx<'info> {
    #[account(
        mut,
        constraint = payer.key() == authority.load()?.admin,
    )]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub authority: AccountLoader<'info, Authority>,
}

pub fn change_fee_collector_handler(
    ctx: Context<FeeCollectorCtx>,
    new_fee_collector: Pubkey,
) -> Result<()> {
    let authority = &mut ctx.accounts.authority.load_mut()?;
    require!(authority.mutable == 1, CustomError::MintIsImmutable);
    authority.fee_collector = new_fee_collector;
    Ok(())
}
