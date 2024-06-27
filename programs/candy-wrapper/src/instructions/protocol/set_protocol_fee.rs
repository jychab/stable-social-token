use anchor_lang::prelude::*;

use crate::state::{ProtocolFeeConfig, PROTOCOL_FEE_CONFIG_SPACE, PROTOCOL_WALLET};
#[derive(Accounts)]
pub struct SetProtocolFeeCtx<'info> {
    #[account(
        mut,
        constraint = payer.key() == PROTOCOL_WALLET,
    )]
    pub payer: Signer<'info>,
    #[account(
        init_if_needed,
        payer = payer,
        seeds = [b"config", PROTOCOL_WALLET.as_ref()],
        bump,
        space = PROTOCOL_FEE_CONFIG_SPACE,
    )]
    pub protocol_fee_config: Account<'info, ProtocolFeeConfig>,
    pub system_program: Program<'info, System>,
}

pub fn set_protocol_fee_handler<'info>(
    ctx: Context<'_, '_, '_, 'info, SetProtocolFeeCtx<'info>>,
    fee_basis_pts: u16,
) -> Result<()> {
    ctx.accounts.protocol_fee_config.bump = ctx.bumps.protocol_fee_config;
    ctx.accounts.protocol_fee_config.fee_basis_pts = fee_basis_pts;
    Ok(())
}
