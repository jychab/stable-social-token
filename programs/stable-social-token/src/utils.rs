use anchor_lang::prelude::*;
use anchor_spl::token_2022::spl_token_2022::{
    extension::{transfer_fee::TransferFeeConfig, BaseStateWithExtensions, StateWithExtensions},
    state,
};

pub fn get_transfer_fee(mint_info: &AccountInfo, amount: u64) -> Result<u64> {
    let mint_data = mint_info.try_borrow_data()?;
    let mint = StateWithExtensions::<state::Mint>::unpack(&mint_data)?;
    let fee = if let Ok(transfer_fee_config) = mint.get_extension::<TransferFeeConfig>() {
        transfer_fee_config
            .calculate_epoch_fee(Clock::get()?.epoch, amount)
            .unwrap()
    } else {
        0
    };
    Ok(fee)
}

pub fn get_withheld_fee(mint_info: &AccountInfo) -> Result<u64> {
    let mint_data = mint_info.try_borrow_data()?;
    let mint = StateWithExtensions::<state::Mint>::unpack(&mint_data)?;
    let extension = mint.get_extension::<TransferFeeConfig>()?;
    let withheld_amount = u64::from(extension.withheld_amount);
    Ok(withheld_amount)
}

fn ceil_div(numerator: u128, denominator: u128) -> Option<u128> {
    numerator
        .checked_add(denominator)?
        .checked_sub(1)?
        .checked_div(denominator)
}

pub fn calculate_fee(amount: u64, transfer_fee_basis_pts: u16) -> u64 {
    let transfer_fee_basis_points = u16::from(transfer_fee_basis_pts) as u128;
    if transfer_fee_basis_points == 0 || amount == 0 {
        0
    } else {
        let numerator = (amount as u128)
            .checked_mul(transfer_fee_basis_points)
            .unwrap();
        let raw_fee = ceil_div(numerator, 10_000)
            .unwrap()
            .try_into() // guaranteed to be okay
            .ok()
            .unwrap();
        raw_fee
    }
}
