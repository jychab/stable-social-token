use anchor_lang::prelude::*;
use anchor_spl::{
    token_2022::{spl_token_2022::instruction::AuthorityType, Token2022},
    token_interface::{
        set_authority, spl_pod::optional_keys::OptionalNonZeroPubkey,
        token_metadata_update_authority, Mint, SetAuthority, TokenInterface,
        TokenMetadataUpdateAuthority,
    },
};

use crate::{error::CustomError, state::Authority};

#[derive(Accounts)]
pub struct ImmutableCtx<'info> {
    #[account(
        mut,
        constraint = payer.key() == authority.load()?.admin,
    )]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub authority: AccountLoader<'info, Authority>,
    #[account(
        mut,
        constraint = mint.key() == authority.load()?.mint @CustomError::IncorrectMint,
    )]
    pub mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        address = Token2022::id()
    )]
    pub token_program_mint: Interface<'info, TokenInterface>,
}

pub fn set_to_immutable_handler(ctx: Context<ImmutableCtx>) -> Result<()> {
    require!(
        ctx.accounts.authority.load()?.mutable == 1,
        CustomError::MintIsImmutable
    );
    ctx.accounts.authority.load_mut()?.mutable = 0;

    let bump = &[ctx.accounts.authority.load()?.bump];
    let mint_key = ctx.accounts.mint.key();
    let seeds: &[&[u8]] = &[b"authority".as_ref(), mint_key.as_ref(), bump];
    let signer_seeds = &[seeds];

    token_metadata_update_authority(
        CpiContext::new(
            ctx.accounts.token_program_mint.to_account_info(),
            TokenMetadataUpdateAuthority {
                token_program_id: ctx.accounts.token_program_mint.to_account_info(),
                metadata: ctx.accounts.mint.to_account_info(),
                current_authority: ctx.accounts.authority.to_account_info(),
                new_authority: ctx.accounts.authority.to_account_info(),
            },
        )
        .with_signer(signer_seeds),
        OptionalNonZeroPubkey(Pubkey::default()),
    )?;
    set_authority(
        CpiContext::new(
            ctx.accounts.token_program_mint.to_account_info(),
            SetAuthority {
                current_authority: ctx.accounts.authority.to_account_info(),
                account_or_mint: ctx.accounts.mint.to_account_info(),
            },
        )
        .with_signer(signer_seeds),
        AuthorityType::MintTokens,
        None,
    )?;
    Ok(())
}
