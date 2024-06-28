use anchor_lang::prelude::*;

declare_id!("2L4rVgwdUgb8KbXyk7VgmrTGtY6XSayaMofF3HMj833E");

mod error;
mod instructions;
mod state;
mod utils;
use instructions::*;

#[program]
pub mod wild_fire {

    use super::*;

    pub fn set_protocol_fee<'info>(
        ctx: Context<'_, '_, '_, 'info, SetProtocolFeeCtx<'info>>,
        fee_basis_pts: u16,
    ) -> Result<()> {
        instructions::set_protocol_fee::set_protocol_fee_handler(ctx, fee_basis_pts)
    }

    pub fn set_to_immutable(ctx: Context<ImmutableCtx>) -> Result<()> {
        instructions::set_to_immutable::set_to_immutable_handler(ctx)
    }

    pub fn change_admin(ctx: Context<AdminCtx>, new_admin: Pubkey) -> Result<()> {
        instructions::change_admin::change_admin_handler(ctx, new_admin)
    }

    pub fn close_account(ctx: Context<CloseAccountCtx>) -> Result<()> {
        instructions::close_account::close_account_handler(ctx)
    }

    pub fn change_fee_collector(
        ctx: Context<FeeCollectorCtx>,
        new_fee_collector: Pubkey,
    ) -> Result<()> {
        instructions::change_fee_collector::change_fee_collector_handler(ctx, new_fee_collector)
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

    pub fn update_mint_metadata(
        ctx: Context<UpdateMintMetadataCtx>,
        field: String,
        value: String,
    ) -> Result<()> {
        instructions::update_mint_metadata::update_mint_metadata_handler(ctx, field, value)
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

    pub fn withdraw_fees<'info>(
        ctx: Context<'_, '_, '_, 'info, WithdrawFeesCtx<'info>>,
    ) -> Result<()> {
        instructions::withdraw_fees::withdraw_fees_handler(ctx)
    }
}
