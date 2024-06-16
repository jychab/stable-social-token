use anchor_lang::prelude::*;
use anchor_spl::{
    token_2022::{spl_token_2022::onchain::invoke_transfer_checked, Token2022},
    token_interface::{
        withdraw_withheld_tokens_from_mint, Mint, TokenAccount, TokenInterface,
        WithdrawWithheldTokensFromMint,
    },
};

use crate::{
    error::CustomError,
    state::{Authority, ProtocolFeeConfig, PROTOCOL_WALLET},
    utils::{calculate_fee, get_withheld_fee},
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

pub fn withdraw_tokens_from_mint_handler<'info>(
    ctx: Context<'_, '_, '_, 'info, WithdrawWithheldTokensFromMintCtx<'info>>,
) -> Result<()> {
    let withheld_amount = get_withheld_fee(&ctx.accounts.mint.to_account_info())?;
    let fee = calculate_fee(
        withheld_amount,
        ctx.accounts.protocol_fee_config.fee_basis_pts,
    );
    let amount_after_fee = withheld_amount.saturating_sub(fee);

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
