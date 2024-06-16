use anchor_lang::prelude::*;
use solana_program::pubkey;

#[cfg(not(feature = "devnet"))]
pub mod stable_coin {
    use solana_program::{pubkey, pubkey::Pubkey};

    pub const USDC: Pubkey = pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
}

#[cfg(feature = "devnet")]
pub mod stable_coin {
    use solana_program::{pubkey, pubkey::Pubkey};

    pub const USDC: Pubkey = pubkey!("4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU");
}

#[account(zero_copy)]
pub struct Authority {
    pub bump: u8,
    pub stable_coin: Pubkey,
    pub mint: Pubkey,
    pub fee_collector: Pubkey,
    pub update_metadata_authority: Pubkey,
}
pub const AUTHORITY_SPACE: usize = 8 + std::mem::size_of::<Authority>();

#[account]
pub struct ProtocolFeeConfig {
    pub bump: u8,
    pub fee_basis_pts: u16,
}

pub const PROTOCOL_FEE_CONFIG_SPACE: usize = 8 + std::mem::size_of::<ProtocolFeeConfig>();

// multi-sig wallet
pub const PROTOCOL_WALLET: Pubkey = pubkey!("FjsF2dg1njhxL9Cv1VezzHropmUDTWRQpcWLANv3jVR2");
