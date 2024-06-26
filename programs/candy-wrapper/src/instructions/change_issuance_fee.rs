use anchor_lang::prelude::*;

use crate::{error::CustomError, state::Authority};

#[derive(Accounts)]
pub struct IssuanceFeeCtx<'info> {
    #[account(
        mut,
        constraint = payer.key() == authority.load()?.admin,
    )]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub authority: AccountLoader<'info, Authority>,
}

pub fn change_issuance_fee_handler(ctx: Context<IssuanceFeeCtx>, fee_basis_pts: u16) -> Result<()> {
    require!(
        fee_basis_pts <= 100,
        CustomError::IssuanceFeeBasisPtsCannotExceed100
    );
    let authority: &mut std::cell::RefMut<Authority> = &mut ctx.accounts.authority.load_mut()?;
    authority.issuance_fee_basis_pts = fee_basis_pts;
    Ok(())
}
