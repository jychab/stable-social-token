use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::Token2022,
    token_interface::{
        mint_to, transfer_checked, Mint, MintTo, TokenAccount, TokenInterface, TransferChecked,
    },
};

use crate::{
    error::CustomError,
    state::{Authority, ProtocolFeeConfig, PROTOCOL_WALLET},
    utils::{calculate_fee, calculate_mint_amount},
};
#[derive(Accounts)]
pub struct IssueMintCtx<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = mint,
        associated_token::authority = payer,
        associated_token::token_program = token_program_mint,
    )]
    pub payer_mint_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        token::mint = base_coin,
        token::authority = payer,
        token::token_program = token_program_base_coin,
    )]
    pub payer_base_coin_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = mint.key() == authority.load()?.mint @CustomError::IncorrectMint,
    )]
    pub mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        constraint = base_coin.key() == authority.load()?.base_coin @CustomError::UnauthorizedBaseCoin,
    )]
    pub base_coin: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = base_coin,
        associated_token::authority = authority,
        associated_token::token_program = token_program_base_coin
    )]
    pub authority_base_coin_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        seeds = [b"authority", mint.key().as_ref()],
        bump = authority.load()?.bump,
    )]
    pub authority: AccountLoader<'info, Authority>,
    #[account(
        mut,
        token::mint = base_coin,
        token::token_program = token_program_base_coin,
        constraint = fee_collector_base_coin_token_account.owner == authority.load()?.fee_collector @CustomError::IncorrectFeeCollector,
    )]
    pub fee_collector_base_coin_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        seeds = [b"config", PROTOCOL_WALLET.as_ref()],
        bump = protocol_fee_config.bump,
    )]
    pub protocol_fee_config: Box<Account<'info, ProtocolFeeConfig>>,
    #[account(
        mut,
        token::mint = base_coin,
        token::token_program = token_program_base_coin,
        constraint = protocol_base_coin_token_account.owner == PROTOCOL_WALLET,
    )]
    pub protocol_base_coin_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        address = Token2022::id()
    )]
    pub token_program_mint: Interface<'info, TokenInterface>,
    pub token_program_base_coin: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn issue_mint_handler<'info>(
    ctx: Context<'_, '_, '_, 'info, IssueMintCtx<'info>>,
    amount: u64,
) -> Result<()> {
    require!(
        ctx.accounts.payer_base_coin_token_account.amount >= amount,
        CustomError::InsufficientAmount
    );

    let mint_key = ctx.accounts.mint.key();
    let seeds: &[&[u8]] = &[
        b"authority",
        mint_key.as_ref(),
        &[ctx.accounts.authority.load()?.bump],
    ];
    let signer = &[seeds];

    let fee = calculate_fee(
        amount,
        ctx.accounts.authority.load()?.issuance_fee_basis_pts,
    );
    let amount_after_fee = amount.saturating_sub(fee);

    let mint_amount = calculate_mint_amount(
        amount_after_fee,
        ctx.accounts.authority.load()?.mint_to_base_ratio,
        ctx.accounts.authority_base_coin_token_account.amount,
        ctx.accounts.mint.supply,
    );

    mint_to(
        CpiContext::new(
            ctx.accounts.token_program_mint.to_account_info(),
            MintTo {
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.payer_mint_token_account.to_account_info(),
                authority: ctx.accounts.authority.to_account_info(),
            },
        )
        .with_signer(signer),
        mint_amount,
    )?;

    if fee > 0 {
        let protocol_fee = calculate_fee(fee, ctx.accounts.protocol_fee_config.fee_basis_pts);
        let amount_after_protocol_fee = fee.saturating_sub(protocol_fee);
        transfer_checked(
            CpiContext::new(
                ctx.accounts.token_program_base_coin.to_account_info(),
                TransferChecked {
                    from: ctx.accounts.payer_base_coin_token_account.to_account_info(),
                    mint: ctx.accounts.base_coin.to_account_info(),
                    to: ctx
                        .accounts
                        .fee_collector_base_coin_token_account
                        .to_account_info(),
                    authority: ctx.accounts.payer.to_account_info(),
                },
            ),
            amount_after_protocol_fee,
            ctx.accounts.base_coin.decimals,
        )?;

        transfer_checked(
            CpiContext::new(
                ctx.accounts.token_program_base_coin.to_account_info(),
                TransferChecked {
                    from: ctx.accounts.payer_base_coin_token_account.to_account_info(),
                    mint: ctx.accounts.base_coin.to_account_info(),
                    to: ctx
                        .accounts
                        .protocol_base_coin_token_account
                        .to_account_info(),
                    authority: ctx.accounts.payer.to_account_info(),
                },
            ),
            protocol_fee,
            ctx.accounts.base_coin.decimals,
        )?;

        ctx.accounts.authority.load_mut()?.fees_collected += amount_after_protocol_fee;
    }

    transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program_base_coin.to_account_info(),
            TransferChecked {
                from: ctx.accounts.payer_base_coin_token_account.to_account_info(),
                mint: ctx.accounts.base_coin.to_account_info(),
                to: ctx
                    .accounts
                    .authority_base_coin_token_account
                    .to_account_info(),
                authority: ctx.accounts.payer.to_account_info(),
            },
        ),
        amount_after_fee,
        ctx.accounts.base_coin.decimals,
    )?;

    Ok(())
}
