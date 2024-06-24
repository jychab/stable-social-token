use anchor_lang::prelude::*;
use anchor_spl::{
    token_2022::Token2022,
    token_interface::{
        initialize_mint, metadata_pointer_initialize, transfer_fee_initialize, InitializeMint,
        MetadataPointerInitialize, Mint, TokenInterface, TransferFeeInitialize,
    },
};

use crate::{
    error::CustomError,
    state::{Authority, AUTHORITY_SPACE},
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CreateMintArgs {
    pub admin: Pubkey,
    pub base_coin: Pubkey,
    pub mint_to_base_ratio: u16,
    pub issuance_fee_basis_pts: u16,
    pub redemption_fee_basis_pts: u16,
    pub fee_collector: Pubkey,
    pub transfer_fee_args: TransferFeeArgs,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct TransferFeeArgs {
    pub fee_basis_pts: u16,
    pub max_fee: u64,
}

#[derive(Accounts)]
#[instruction(args: CreateMintArgs)]
pub struct CreateMintCtx<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        mut,
        owner = token_program_2022.key(),
    )]
    /// CHECK: Mint to be created
    pub mint: AccountInfo<'info>,
    #[account(
        init,
        space = AUTHORITY_SPACE,
        payer = payer,
        seeds = [b"authority", mint.key().as_ref()],
        bump,
    )]
    pub authority: AccountLoader<'info, Authority>,
    #[account(
        constraint = base_coin.key() == args.base_coin @CustomError::UnauthorizedBaseCoin,
    )]
    pub base_coin: InterfaceAccount<'info, Mint>,
    #[account(
        address = Token2022::id() @CustomError::IncorrectTokenProgram,
    )]
    pub token_program_2022: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn create_mint_handler(ctx: Context<CreateMintCtx>, args: CreateMintArgs) -> Result<()> {
    let authority = &mut ctx.accounts.authority.load_init()?;
    authority.bump = ctx.bumps.authority;
    authority.base_coin = args.base_coin;
    authority.mint = ctx.accounts.mint.key();
    authority.mint_to_base_ratio = args.mint_to_base_ratio;
    authority.fee_collector = args.fee_collector;
    authority.admin = args.admin;
    authority.issuance_fee_basis_pts = args.issuance_fee_basis_pts;
    authority.redemption_fee_basis_pts = args.redemption_fee_basis_pts;

    require!(
        args.mint_to_base_ratio > 0,
        CustomError::MintRatioCannotBeZero
    );

    // initialize transfer fee
    transfer_fee_initialize(
        CpiContext::new(
            ctx.accounts.token_program_2022.to_account_info(),
            TransferFeeInitialize {
                token_program_id: ctx.accounts.token_program_2022.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
            },
        ),
        Some(&args.admin),
        Some(&ctx.accounts.authority.key()),
        args.transfer_fee_args.fee_basis_pts,
        args.transfer_fee_args.max_fee,
    )?;

    // initialize mint metadata pointer
    metadata_pointer_initialize(
        CpiContext::new(
            ctx.accounts.token_program_2022.to_account_info(),
            MetadataPointerInitialize {
                token_program_id: ctx.accounts.token_program_2022.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
            },
        ),
        Some(args.admin),
        Some(ctx.accounts.mint.key()),
    )?;

    // intialize mint
    initialize_mint(
        CpiContext::new(
            ctx.accounts.token_program_2022.to_account_info(),
            InitializeMint {
                mint: ctx.accounts.mint.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
        ),
        ctx.accounts.base_coin.decimals,
        &ctx.accounts.authority.key(),
        None,
    )?;

    Ok(())
}
