use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::Token,
    token_2022::Token2022,
    token_interface::{
        burn, transfer_checked, Burn, Mint, TokenAccount, TokenInterface, TransferChecked,
    },
};

use crate::{
    error::CustomError,
    state::{stable_coin, Authority},
    utils::get_transfer_fee,
};
#[derive(Accounts)]
pub struct RedeemStableCoinCtx<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        mut,
        token::mint = mint,
        token::authority = payer
    )]
    pub payer_mint_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = stable_coin,
        associated_token::authority = payer,
        associated_token::token_program = token_program,
    )]
    pub payer_stable_coin_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = mint.key() == authority.load()?.mint @CustomError::IncorrectMint,
    )]
    pub mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        address = stable_coin::USDC @CustomError::UnauthorizedStableCoin,
    )]
    pub stable_coin: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        mut,
        token::mint = stable_coin,
        token::authority = authority,
        token::token_program = token_program,
    )]
    pub authority_stable_coin_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        seeds = [b"authority", mint.key().as_ref()],
        bump = authority.load()?.bump,
    )]
    pub authority: AccountLoader<'info, Authority>,
    #[account(
        mut,
        token::mint = stable_coin,
        token::token_program = token_program,
        constraint = fee_collector_stable_coin_token_account.owner == authority.load()?.fee_collector @CustomError::IncorrectFeeCollector,
    )]
    pub fee_collector_stable_coin_token_account: Option<Box<InterfaceAccount<'info, TokenAccount>>>,
    #[account(
        address = Token2022::id()
    )]
    pub token_program_2022: Interface<'info, TokenInterface>,
    #[account(
        address = Token::id()
    )]
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn redeem_stablecoin_handler<'info>(
    ctx: Context<'_, '_, '_, 'info, RedeemStableCoinCtx<'info>>,
    amount: u64,
) -> Result<()> {
    require!(
        ctx.accounts.payer_mint_token_account.amount >= amount,
        CustomError::InsufficientAmount
    );

    let fee = get_transfer_fee(&ctx.accounts.mint.to_account_info(), amount)?;

    let mint_key = ctx.accounts.mint.key();
    let seeds = &[
        b"authority",
        mint_key.as_ref(),
        &[ctx.accounts.authority.load()?.bump],
    ];
    let signer = &[&seeds[..]];

    if let Some(fee_collector_token_account) = &ctx.accounts.fee_collector_stable_coin_token_account
    {
        if fee > 0 {
            transfer_checked(
                CpiContext::new(
                    ctx.accounts.token_program.to_account_info(),
                    TransferChecked {
                        from: ctx
                            .accounts
                            .authority_stable_coin_token_account
                            .to_account_info(),
                        mint: ctx.accounts.stable_coin.to_account_info(),
                        to: fee_collector_token_account.to_account_info(),
                        authority: ctx.accounts.authority.to_account_info(),
                    },
                )
                .with_signer(signer),
                fee,
                ctx.accounts.stable_coin.decimals,
            )?;
        }
    } else {
        require!(fee == 0, CustomError::MissingFeeCollector);
    }

    transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx
                    .accounts
                    .authority_stable_coin_token_account
                    .to_account_info(),
                mint: ctx.accounts.stable_coin.to_account_info(),
                to: ctx
                    .accounts
                    .payer_stable_coin_token_account
                    .to_account_info(),
                authority: ctx.accounts.authority.to_account_info(),
            },
        )
        .with_signer(signer),
        amount.saturating_sub(fee),
        ctx.accounts.stable_coin.decimals,
    )?;

    burn(
        CpiContext::new(
            ctx.accounts.token_program_2022.to_account_info(),
            Burn {
                mint: ctx.accounts.mint.to_account_info(),
                from: ctx.accounts.payer_mint_token_account.to_account_info(),
                authority: ctx.accounts.payer.to_account_info(),
            },
        ),
        amount,
    )?;

    Ok(())
}
