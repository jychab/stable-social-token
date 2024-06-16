use anchor_lang::prelude::*;
use anchor_spl::{
    token_2022::{
        spl_token_2022::{
            extension::{
                transfer_fee::TransferFeeConfig, BaseStateWithExtensions, StateWithExtensions,
            },
            onchain::invoke_transfer_checked,
            state,
        },
        Token2022,
    },
    token_interface::{
        withdraw_withheld_tokens_from_mint, Mint, TokenAccount, TokenInterface,
        WithdrawWithheldTokensFromMint,
    },
};

use crate::{
    error::CustomError,
    state::{Authority, ProtocolFeeConfig, PROTOCOL_WALLET},
};
#[derive(Accounts)]
pub struct WithdrawWithheldTokensFromMintCtx<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        mut,
        constraint = mint.key() == authority.load()?.mint @CustomError::IncorrectMint,
    )]
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(
        seeds = [b"authority", mint.key().as_ref()],
        bump = authority.load()?.bump,
    )]
    pub authority: AccountLoader<'info, Authority>,
    #[account(
        mut,
        token::mint = mint,
        token::token_program = token_program_2022,
        constraint = fee_collector_mint_token_account.owner == authority.load()?.fee_collector @CustomError::IncorrectFeeCollector,
    )]
    pub fee_collector_mint_token_account: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        token::mint = mint,
        token::authority = protocol_fee_config,
        token::token_program = token_program_2022,
    )]
    pub protocol_config_mint_token_account: InterfaceAccount<'info, TokenAccount>,
    #[account(
        seeds = [b"config", PROTOCOL_WALLET.as_ref()],
        bump = protocol_fee_config.bump,
    )]
    pub protocol_fee_config: Account<'info, ProtocolFeeConfig>,
    #[account(
        address = Token2022::id()
    )]
    pub token_program_2022: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

fn ceil_div(numerator: u128, denominator: u128) -> Option<u128> {
    numerator
        .checked_add(denominator)?
        .checked_sub(1)?
        .checked_div(denominator)
}

fn calculate_fee(amount: u64, transfer_fee_basis_pts: u16) -> u64 {
    let transfer_fee_basis_points = u16::from(transfer_fee_basis_pts) as u128;
    if transfer_fee_basis_points == 0 || amount == 0 {
        0
    } else {
        let numerator = (amount as u128)
            .checked_mul(transfer_fee_basis_points)
            .unwrap();
        let raw_fee = ceil_div(numerator, 10_000)
            .unwrap()
            .try_into() // guaranteed to be okay
            .ok()
            .unwrap();
        raw_fee
    }
}

pub fn withdraw_tokens_from_mint_handler<'info>(
    ctx: Context<'_, '_, '_, 'info, WithdrawWithheldTokensFromMintCtx<'info>>,
) -> Result<()> {
    let mint_info = ctx.accounts.mint.to_account_info();
    let mint_data = mint_info.data.borrow();
    let mint = StateWithExtensions::<state::Mint>::unpack(&mint_data)?;
    let extension = mint.get_extension::<TransferFeeConfig>()?;

    let withheld_amount = u64::from(extension.withheld_amount);
    let fee = calculate_fee(
        withheld_amount,
        ctx.accounts.protocol_fee_config.fee_basis_pts,
    );
    let amount_after_fee = withheld_amount.saturating_sub(fee);
    drop(mint_data);

    let mint_key = ctx.accounts.mint.key();
    let seeds = &[
        b"authority",
        mint_key.as_ref(),
        &[ctx.accounts.authority.load()?.bump],
    ];
    let signer = &[&seeds[..]];

    withdraw_withheld_tokens_from_mint(
        CpiContext::new(
            ctx.accounts.token_program_2022.to_account_info(),
            WithdrawWithheldTokensFromMint {
                token_program_id: ctx.accounts.token_program_2022.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                destination: ctx
                    .accounts
                    .protocol_config_mint_token_account
                    .to_account_info(),
                authority: ctx.accounts.authority.to_account_info(),
            },
        )
        .with_signer(signer),
    )?;

    let seeds = &[
        b"config",
        PROTOCOL_WALLET.as_ref(),
        &[ctx.accounts.protocol_fee_config.bump],
    ];
    let signer = &[&seeds[..]];

    invoke_transfer_checked(
        &ctx.accounts.token_program_2022.key,
        ctx.accounts
            .protocol_config_mint_token_account
            .to_account_info(),
        ctx.accounts.mint.to_account_info(),
        ctx.accounts
            .fee_collector_mint_token_account
            .to_account_info(),
        ctx.accounts.protocol_fee_config.to_account_info(),
        ctx.remaining_accounts,
        amount_after_fee,
        ctx.accounts.mint.decimals,
        signer,
    )?;

    Ok(())
}
