use anchor_lang::prelude::*;

use crate::{error::CustomError, state::Authority};

#[derive(Accounts)]
pub struct AdminCtx<'info> {
    #[account(
        mut,
        constraint = payer.key() == authority.load()?.admin,
    )]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub authority: AccountLoader<'info, Authority>,
}

pub fn change_admin_handler(ctx: Context<AdminCtx>, new_admin: Pubkey) -> Result<()> {
    let authority = &mut ctx.accounts.authority.load_mut()?;
    require!(authority.mutable == 1, CustomError::MintIsImmutable);
    authority.admin = new_admin;
    Ok(())
}
