use anchor_lang::prelude::*;

declare_id!("APP6sVJA6zKhnxVfTdEvkvHN9xGUNYxYZy3nQ6DePEAX");

mod error;
mod instructions;
mod state;
mod utils;

use instructions::*;
#[program]
pub mod stable_social_token {

    use super::*;

    pub fn set_protocol_fee<'info>(
        ctx: Context<'_, '_, '_, 'info, SetProtocolFeeCtx<'info>>,
        fee_basis_pts: u16,
    ) -> Result<()> {
        instructions::set_protocol_fee::set_protocol_fee_handler(ctx, fee_basis_pts)
    }

    pub fn create_mint(ctx: Context<CreateMintCtx>, args: CreateMintArgs) -> Result<()> {
        instructions::create_mint::create_mint_handler(ctx, args)
    }

    pub fn create_mint_metadata(
        ctx: Context<CreateMintMetadataCtx>,
        lamports: u64,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<()> {
        instructions::create_mint_metadata::create_mint_metadata_handler(
            ctx, lamports, name, symbol, uri,
        )
    }

    pub fn issue_mint<'info>(
        ctx: Context<'_, '_, '_, 'info, IssueMintCtx<'info>>,
        amount: u64,
    ) -> Result<()> {
        instructions::issue_mint::issue_mint_handler(ctx, amount)
    }

    pub fn redeem_basecoin<'info>(
        ctx: Context<'_, '_, '_, 'info, RedeemBaseCoinCtx<'info>>,
        amount: u64,
    ) -> Result<()> {
        instructions::redeem_basecoin::redeem_basecoin_handler(ctx, amount)
    }

    pub fn withdraw_fees<'info>(
        ctx: Context<'_, '_, '_, 'info, WithdrawFeesCtx<'info>>,
    ) -> Result<()> {
        instructions::withdraw_fees::withdraw_fees_handler(ctx)
    }
}
