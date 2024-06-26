use anchor_lang::prelude::*;

declare_id!("4iKL2rCj42ZmvtzPQBadTcNndneJR2FRtJSmc8XzXAp9");

mod error;
mod instructions;
mod state;
mod utils;

use instructions::*;
#[program]
pub mod candy_wrapper {

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

    pub fn change_transfer_fee<'info>(
        ctx: Context<'_, '_, '_, 'info, TransferFeeCtx<'info>>,
        fee_basis_pts: u16,
        max_fee: u64,
    ) -> Result<()> {
        instructions::change_transfer_fee::change_transfer_fee_handler(ctx, fee_basis_pts, max_fee)
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

    pub fn change_issuance_fee(ctx: Context<IssuanceFeeCtx>, fee_basis_pts: u16) -> Result<()> {
        instructions::change_issuance_fee::change_issuance_fee_handler(ctx, fee_basis_pts)
    }

    pub fn change_redemption_fee(ctx: Context<RedemptionFeeCtx>, fee_basis_pts: u16) -> Result<()> {
        instructions::change_redemption_fee::change_redemption_fee_handler(ctx, fee_basis_pts)
    }
}
