use anchor_lang::prelude::*;

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
