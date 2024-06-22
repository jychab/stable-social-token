use anchor_lang::prelude::*;
use solana_program::pubkey;

#[account(zero_copy)]
pub struct Authority {
    pub fees_collected: u64,
    pub mint_to_base_ratio: u16,
    pub padding: [u8; 5],
    pub bump: u8,
    pub base_coin: Pubkey,
    pub mint: Pubkey,
    pub fee_collector: Pubkey,
    pub admin: Pubkey, // have the ability to change transfer fees, change transfer hook program Id & change metadata
}
pub const AUTHORITY_SPACE: usize = 8 + std::mem::size_of::<Authority>();

#[account]
pub struct ProtocolFeeConfig {
    pub bump: u8,
    pub fee_basis_pts: u16,
}

pub const PROTOCOL_FEE_CONFIG_SPACE: usize = 8 + std::mem::size_of::<ProtocolFeeConfig>();

// multi-sig wallet
pub const PROTOCOL_WALLET: Pubkey = pubkey!("G6kBnedts6uAivtY72ToaFHBs1UVbT9udiXmQZgMEjoF");
