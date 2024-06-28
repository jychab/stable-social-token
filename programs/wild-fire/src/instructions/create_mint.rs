use anchor_lang::{
    prelude::*,
    system_program::{self, Transfer},
};
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::Token2022,
    token_interface::{
        initialize_mint, metadata_pointer_initialize, transfer_fee_initialize, InitializeMint,
        MetadataPointerInitialize, TokenInterface, TransferFeeInitialize,
    },
};

use crate::state::{Authority, AUTHORITY_SPACE, PROTOCOL_WALLET};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CreateMintArgs {
    pub admin: Pubkey,
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
        owner = token_program_mint.key(),
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
        address = PROTOCOL_WALLET,
    )]
    /// CHECK:
    pub protocol_wallet: AccountInfo<'info>,
    #[account(
        address = Token2022::id(),
    )]
    pub token_program_mint: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn create_mint_handler(ctx: Context<CreateMintCtx>, args: CreateMintArgs) -> Result<()> {
    let authority = &mut ctx.accounts.authority.load_init()?;
    authority.bump = ctx.bumps.authority;
    authority.mint = ctx.accounts.mint.key();
    authority.fee_collector = args.fee_collector;
    authority.admin = args.admin;
    authority.mutable = 1;

    // initialize transfer fee
    transfer_fee_initialize(
        CpiContext::new(
            ctx.accounts.token_program_mint.to_account_info(),
            TransferFeeInitialize {
                token_program_id: ctx.accounts.token_program_mint.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
            },
        ),
        Some(&ctx.accounts.authority.key()),
        Some(&ctx.accounts.authority.key()),
        args.transfer_fee_args.fee_basis_pts,
        args.transfer_fee_args.max_fee,
    )?;

    // initialize mint metadata pointer
    metadata_pointer_initialize(
        CpiContext::new(
            ctx.accounts.token_program_mint.to_account_info(),
            MetadataPointerInitialize {
                token_program_id: ctx.accounts.token_program_mint.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
            },
        ),
        Some(ctx.accounts.authority.key()),
        Some(ctx.accounts.mint.key()),
    )?;

    // intialize mint
    initialize_mint(
        CpiContext::new(
            ctx.accounts.token_program_mint.to_account_info(),
            InitializeMint {
                mint: ctx.accounts.mint.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
        ),
        6,
        &ctx.accounts.authority.key(),
        None,
    )?;

    system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: ctx.accounts.payer.to_account_info(),
                to: ctx.accounts.protocol_wallet.to_account_info(),
            },
        ),
        100000000, // 0.1 SOL
    )?;

    Ok(())
}
