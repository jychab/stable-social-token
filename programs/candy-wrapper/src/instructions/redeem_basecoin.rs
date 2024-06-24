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
    state::Authority,
    utils::{calculate_base_coin_amount, calculate_fee},
};
#[derive(Accounts)]
pub struct RedeemBaseCoinCtx<'info> {
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
        associated_token::mint = base_coin,
        associated_token::authority = payer,
        associated_token::token_program = token_program,
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
        mut,
        token::mint = base_coin,
        token::authority = authority,
        token::token_program = token_program,
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
        token::token_program = token_program,
        constraint = fee_collector_base_coin_token_account.owner == authority.load()?.fee_collector @CustomError::IncorrectFeeCollector,
    )]
    pub fee_collector_base_coin_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
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

pub fn redeem_basecoin_handler<'info>(
    ctx: Context<'_, '_, '_, 'info, RedeemBaseCoinCtx<'info>>,
    amount: u64,
) -> Result<()> {
    require!(
        ctx.accounts.payer_mint_token_account.amount >= amount,
        CustomError::InsufficientAmount
    );

    let mint_key = ctx.accounts.mint.key();
    let seeds: &[&[u8]] = &[
        b"authority",
        mint_key.as_ref(),
        &[ctx.accounts.authority.load()?.bump],
    ];
    let signer = &[seeds];

    let base_coin_amount = calculate_base_coin_amount(
        amount,
        ctx.accounts.authority_base_coin_token_account.amount,
        ctx.accounts.mint.supply,
    );

    let fee = calculate_fee(
        base_coin_amount,
        ctx.accounts.authority.load()?.redemption_fee_basis_pts,
    );

    let amount_after_fee = base_coin_amount.saturating_sub(fee);

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

    if fee > 0 {
        ctx.accounts.authority.load_mut()?.fees_collected += fee;
        transfer_checked(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                    from: ctx
                        .accounts
                        .authority_base_coin_token_account
                        .to_account_info(),
                    mint: ctx.accounts.base_coin.to_account_info(),
                    to: ctx
                        .accounts
                        .fee_collector_base_coin_token_account
                        .to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                },
            )
            .with_signer(signer),
            fee,
            ctx.accounts.base_coin.decimals,
        )?;
    }

    transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx
                    .accounts
                    .authority_base_coin_token_account
                    .to_account_info(),
                mint: ctx.accounts.base_coin.to_account_info(),
                to: ctx.accounts.payer_base_coin_token_account.to_account_info(),
                authority: ctx.accounts.authority.to_account_info(),
            },
        )
        .with_signer(signer),
        amount_after_fee,
        ctx.accounts.base_coin.decimals,
    )?;

    Ok(())
}
