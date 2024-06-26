use anchor_lang::prelude::*;
use anchor_spl::token_2022::spl_token_2022::{
    extension::{transfer_fee::TransferFeeConfig, BaseStateWithExtensions, StateWithExtensions},
    state,
};

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
    let transfer_fee_basis_points = transfer_fee_basis_pts as u128;
    if transfer_fee_basis_points == 0 || amount == 0 {
        0
    } else {
        let numerator = (amount as u128)
            .checked_mul(transfer_fee_basis_points)
            .unwrap();
        ceil_div(numerator, 10_000)
            .unwrap()
            .try_into()
            .ok()
            .unwrap()
    }
}

pub fn calculate_mint_amount(
    base_coin_amount: u64,
    mint_to_base_ratio: u16,
    authority_base_coin_amount: u64,
    mint_supply_amount: u64,
) -> u64 {
    if mint_supply_amount > 0 {
        (base_coin_amount as u128)
            .checked_mul(mint_supply_amount as u128)
            .unwrap()
            .checked_div(authority_base_coin_amount as u128)
            .unwrap()
            .try_into()
            .ok()
            .unwrap()
    } else {
        (base_coin_amount as u128)
            .checked_mul(mint_to_base_ratio as u128)
            .unwrap()
            .try_into()
            .ok()
            .unwrap()
    }
}

pub fn calculate_base_coin_amount(
    mint_amount: u64,
    authority_base_coin_amount: u64,
    mint_supply_amount: u64,
) -> u64 {
    (mint_amount as u128)
        .checked_mul(authority_base_coin_amount as u128)
        .unwrap()
        .checked_div(mint_supply_amount as u128)
        .unwrap()
        .try_into()
        .ok()
        .unwrap()
}
