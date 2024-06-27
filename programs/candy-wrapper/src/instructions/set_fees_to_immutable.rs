use anchor_lang::prelude::*;

use crate::{error::CustomError, state::Authority};

#[derive(Accounts)]
pub struct ImmutableCtx<'info> {
    #[account(
        mut,
        constraint = payer.key() == authority.load()?.admin,
    )]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub authority: AccountLoader<'info, Authority>,
}

pub fn set_fees_to_immutable_handler(ctx: Context<ImmutableCtx>) -> Result<()> {
    let authority: &mut std::cell::RefMut<Authority> = &mut ctx.accounts.authority.load_mut()?;
    require!(authority.mutable == 1, CustomError::MintIsImmutable);
    authority.mutable = 0;
    Ok(())
}
