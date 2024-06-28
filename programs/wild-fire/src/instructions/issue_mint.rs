use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::Token2022,
    token_interface::{mint_to, Mint, MintTo, TokenAccount, TokenInterface},
};

use crate::{error::CustomError, state::Authority};
#[derive(Accounts)]
pub struct IssueMintCtx<'info> {
    #[account(
        mut,
        constraint = payer.key() == authority.load()?.admin
    )]
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
        constraint = mint.key() == authority.load()?.mint @CustomError::IncorrectMint,
    )]
    pub mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        mut,
        seeds = [b"authority", mint.key().as_ref()],
        bump = authority.load()?.bump,
    )]
    pub authority: AccountLoader<'info, Authority>,
    #[account(
        address = Token2022::id()
    )]
    pub token_program_mint: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn issue_mint_handler<'info>(
    ctx: Context<'_, '_, '_, 'info, IssueMintCtx<'info>>,
    amount: u64,
) -> Result<()> {
    require!(
        ctx.accounts.authority.load()?.mutable == 1,
        CustomError::MintIsImmutable
    );
    let mint_key = ctx.accounts.mint.key();
    let seeds: &[&[u8]] = &[
        b"authority",
        mint_key.as_ref(),
        &[ctx.accounts.authority.load()?.bump],
    ];
    let signer = &[seeds];

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
        amount,
    )?;

    Ok(())
}
