use anchor_lang::prelude::*;
use anchor_spl::{
    token_2022::Token2022,
    token_interface::{
        token_metadata_update_field, Mint, TokenInterface, TokenMetadataUpdateField,
    },
};

use crate::{error::CustomError, state::Authority};

#[derive(Accounts)]
pub struct UpdateMintMetadataCtx<'info> {
    #[account(
        mut,
        constraint = payer.key() == authority.load()?.admin,
    )]
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
        address = Token2022::id()
    )]
    pub token_program_mint: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

pub fn update_mint_metadata_handler(
    ctx: Context<UpdateMintMetadataCtx>,
    field: String,
    value: String,
) -> Result<()> {
    let bump = &[ctx.accounts.authority.load()?.bump];
    let mint_key = ctx.accounts.mint.key();
    let seeds: &[&[u8]] = &[b"authority".as_ref(), mint_key.as_ref(), bump];
    let signer_seeds = &[seeds];

    token_metadata_update_field(
        CpiContext::new(
            ctx.accounts.token_program_mint.to_account_info(),
            TokenMetadataUpdateField {
                token_program_id: ctx.accounts.token_program_mint.to_account_info(),
                metadata: ctx.accounts.mint.to_account_info(),
                update_authority: ctx.accounts.authority.to_account_info(),
            },
        )
        .with_signer(signer_seeds),
        anchor_spl::token_2022_extensions::spl_token_metadata_interface::state::Field::Key(field),
        value,
    )?;

    Ok(())
}
