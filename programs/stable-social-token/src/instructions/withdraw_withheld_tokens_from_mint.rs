use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken, token_2022::Token2022, token_interface::{withdraw_withheld_tokens_from_mint, Mint, TokenAccount, TokenInterface, WithdrawWithheldTokensFromMint}
};

use crate::{error::CustomError, state::Authority};
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
        init_if_needed,
        payer = payer, 
        associated_token::mint = mint,
        associated_token::authority = fee_collector,
        associated_token::token_program = token_program_2022,
    )]
    pub fee_collector_mint_token_account: InterfaceAccount<'info, TokenAccount>,
    #[account(
        constraint = fee_collector.key() == authority.load()?.fee_collector @CustomError::IncorrectFeeCollector,
    )]
    /// CHECK: Checked by constraint
    pub fee_collector: AccountInfo<'info>,
    #[account(
        address = Token2022::id()
    )]
    pub token_program_2022: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn withdraw_tokens_from_mint_handler<'info>(
    ctx: Context<'_, '_, '_, 'info, WithdrawWithheldTokensFromMintCtx<'info>>,
) -> Result<()> {
    
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
                destination: ctx.accounts.fee_collector_mint_token_account.to_account_info(),
                authority: ctx.accounts.authority.to_account_info(),
            },
        )
        .with_signer(signer),
    )?;
    Ok(())
}
