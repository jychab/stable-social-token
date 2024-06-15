use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::Token,
    token_2022::Token2022,
    token_interface::{
        mint_to, transfer_checked, Mint, MintTo, TokenAccount, TokenInterface, TransferChecked,
    },
};

use crate::{
    error::CustomError,
    state::{stable_coin, Authority},
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
        associated_token::token_program = token_program_2022,
    )]
    pub payer_mint_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        token::mint = stable_coin,
        token::authority = payer,
        token::token_program = token_program,
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
        init_if_needed,
        payer = payer,
        associated_token::mint = stable_coin,
        associated_token::authority = authority,
        associated_token::token_program = token_program
    )]
    pub authority_stable_coin_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        seeds = [b"authority", mint.key().as_ref()],
        bump = authority.load()?.bump,
    )]
    pub authority: AccountLoader<'info, Authority>,
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

pub fn issue_mint_handler<'info>(
    ctx: Context<'_, '_, '_, 'info, IssueMintCtx<'info>>,
    amount: u64,
) -> Result<()> {
    let mint_key = ctx.accounts.mint.key();
    let seeds = &[
        b"authority",
        mint_key.as_ref(),
        &[ctx.accounts.authority.load()?.bump],
    ];
    let signer = &[&seeds[..]];
    transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx
                    .accounts
                    .payer_stable_coin_token_account
                    .to_account_info(),
                mint: ctx.accounts.stable_coin.to_account_info(),
                to: ctx
                    .accounts
                    .authority_stable_coin_token_account
                    .to_account_info(),
                authority: ctx.accounts.payer.to_account_info(),
            },
        ),
        amount,
        ctx.accounts.stable_coin.decimals,
    )?;

    mint_to(
        CpiContext::new(
            ctx.accounts.token_program_2022.to_account_info(),
            MintTo {
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.payer_mint_token_account.to_account_info(),
                authority: ctx.accounts.authority.to_account_info(),
            },
        )
        .with_signer(signer),
        amount,
    )?;

    Ok(())
}
