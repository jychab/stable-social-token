use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::Token,
    token_2022::Token2022,
    token_interface::{
        burn, transfer_checked, withdraw_withheld_tokens_from_mint, Burn, Mint, TokenAccount,
        TokenInterface, TransferChecked, WithdrawWithheldTokensFromMint,
    },
};

use crate::{
    error::CustomError,
    state::{stable_coin, Authority, ProtocolFeeConfig, PROTOCOL_WALLET},
    utils::{calculate_fee, get_withheld_fee},
};
#[derive(Accounts)]
pub struct WithdrawFeesCtx<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        mut,
        constraint = mint.key() == authority.load()?.mint @CustomError::IncorrectMint,
    )]
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(
        address = stable_coin::USDC @CustomError::UnauthorizedStableCoin,
    )]
    pub stable_coin: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        seeds = [b"authority", mint.key().as_ref()],
        bump = authority.load()?.bump,
    )]
    pub authority: AccountLoader<'info, Authority>,
    #[account(
        seeds = [b"config", PROTOCOL_WALLET.as_ref()],
        bump = protocol_fee_config.bump,
    )]
    pub protocol_fee_config: Account<'info, ProtocolFeeConfig>,
    #[account(
        address = PROTOCOL_WALLET
    )]
    /// CHECK: Checked by address
    pub protocol_wallet: AccountInfo<'info>,

    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = mint,
        associated_token::authority = authority,
        associated_token::token_program = token_program_2022,
    )]
    pub authority_mint_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        token::mint = stable_coin,
        token::authority = authority,
        token::token_program = token_program,
    )]
    pub authority_stable_coin_token_account: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        token::mint = stable_coin,
        token::authority = protocol_wallet,
        token::token_program = token_program,
    )]
    pub protocol_stable_coin_token_account: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        token::mint = stable_coin,
        token::token_program = token_program,
        constraint = fee_collector_stable_coin_token_account.owner == authority.load()?.fee_collector @CustomError::IncorrectFeeCollector,
    )]
    pub fee_collector_stable_coin_token_account: InterfaceAccount<'info, TokenAccount>,

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

pub fn withdraw_fees_handler<'info>(
    ctx: Context<'_, '_, '_, 'info, WithdrawFeesCtx<'info>>,
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
                destination: ctx.accounts.authority_mint_token_account.to_account_info(),
                authority: ctx.accounts.authority.to_account_info(),
            },
        )
        .with_signer(signer),
    )?;

    burn(
        CpiContext::new(
            ctx.accounts.token_program_2022.to_account_info(),
            Burn {
                mint: ctx.accounts.mint.to_account_info(),
                from: ctx.accounts.authority_mint_token_account.to_account_info(),
                authority: ctx.accounts.authority.to_account_info(),
            },
        )
        .with_signer(signer),
        withheld_amount,
    )?;

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
                    .protocol_stable_coin_token_account
                    .to_account_info(),
                authority: ctx.accounts.authority.to_account_info(),
            },
        )
        .with_signer(signer),
        fee,
        ctx.accounts.stable_coin.decimals,
    )?;

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
                    .fee_collector_stable_coin_token_account
                    .to_account_info(),
                authority: ctx.accounts.authority.to_account_info(),
            },
        )
        .with_signer(signer),
        amount_after_fee,
        ctx.accounts.stable_coin.decimals,
    )?;
    ctx.accounts.authority.load_mut()?.fees_collected += amount_after_fee;

    Ok(())
}
