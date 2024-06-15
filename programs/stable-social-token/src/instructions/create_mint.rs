use anchor_lang::prelude::*;
use anchor_spl::{
    token_2022::Token2022,
    token_interface::{
        initialize_mint, metadata_pointer_initialize, permanent_delegate_initialize,
        transfer_fee_initialize, transfer_hook_initialize, InitializeMint,
        MetadataPointerInitialize, Mint, PermanentDelegateInitialize, TokenInterface,
        TransferFeeInitialize, TransferHookInitialize,
    },
};

use crate::{
    error::CustomError,
    state::{stable_coin, Authority, AUTHORITY_SPACE},
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CreateMintArgs {
    pub random_key: Pubkey,
    pub size: u16,
    pub transfer_fee_args: Option<TransferFeeArgs>,
    pub transfer_hook_args: Option<TransferHookArgs>,
    pub permanent_delegate: Option<Pubkey>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct TransferFeeArgs {
    pub fee_basis_pts: u16,
    pub max_fee: u64,
    pub fee_collector: Pubkey, // pubkey that all fees will be withdrawn to
    pub transfer_fee_config_authority: Pubkey, // authority allowed to change transfer fee
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct TransferHookArgs {
    pub transfer_hook_program_id: Pubkey,
    pub transfer_hook_authority: Pubkey, // authority allowed to change transfer_hook_program id
}

#[derive(Accounts)]
#[instruction(args: CreateMintArgs)]
pub struct CreateMintCtx<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        init,
        payer = payer,
        seeds = [b"mint", args.random_key.as_ref()],
        bump,
        owner = token_program_2022.key(),
        space = args.size.into(),
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
        address = stable_coin::USDC @CustomError::UnauthorizedStableCoin,
    )]
    pub stable_coin: InterfaceAccount<'info, Mint>,
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
    authority.stable_coin = ctx.accounts.stable_coin.key();
    authority.mint = ctx.accounts.mint.key();
    authority.update_metadata_authority = ctx.accounts.payer.key();

    if let Some(transfer_fee_args) = args.transfer_fee_args {
        authority.fee_collector = transfer_fee_args.fee_collector;
        // initialize transfer fee
        transfer_fee_initialize(
            CpiContext::new(
                ctx.accounts.token_program_2022.to_account_info(),
                TransferFeeInitialize {
                    token_program_id: ctx.accounts.token_program_2022.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                },
            ),
            Some(&transfer_fee_args.transfer_fee_config_authority),
            Some(&ctx.accounts.authority.key()),
            transfer_fee_args.fee_basis_pts,
            transfer_fee_args.max_fee,
        )?;
    }

    if let Some(transfer_hook_args) = args.transfer_hook_args {
        // initialize transfer hook
        transfer_hook_initialize(
            CpiContext::new(
                ctx.accounts.token_program_2022.to_account_info(),
                TransferHookInitialize {
                    token_program_id: ctx.accounts.token_program_2022.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                },
            ),
            Some(transfer_hook_args.transfer_hook_authority),
            Some(transfer_hook_args.transfer_hook_program_id),
        )?;
    }

    if let Some(permanent_delegate) = args.permanent_delegate {
        permanent_delegate_initialize(
            CpiContext::new(
                ctx.accounts.token_program_2022.to_account_info(),
                PermanentDelegateInitialize {
                    token_program_id: ctx.accounts.token_program_2022.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                },
            ),
            &permanent_delegate,
        )?;
    }

    // initialize mint metadata pointer
    metadata_pointer_initialize(
        CpiContext::new(
            ctx.accounts.token_program_2022.to_account_info(),
            MetadataPointerInitialize {
                token_program_id: ctx.accounts.token_program_2022.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
            },
        ),
        Some(ctx.accounts.payer.key()),
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
        ctx.accounts.stable_coin.decimals,
        &ctx.accounts.authority.key(),
        None,
    )?;

    Ok(())
}
