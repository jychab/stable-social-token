use anchor_lang::prelude::*;
use solana_program::pubkey;

#[account(zero_copy)]
pub struct Authority {
    pub fees_collected: u64,
    pub bump: u8,
    pub mutable: u8,
    pub padding: [u8; 6],
    pub mint: Pubkey,
    pub fee_collector: Pubkey,
    pub admin: Pubkey,
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
