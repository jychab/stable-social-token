use anchor_lang::prelude::*;
// An enum for custom error codes
#[error_code]
pub enum CustomError {
    IncorrectUpdateAuthority,
    IncorrectMint,
    IncorrectFeeCollector,
    IncorrectTokenProgram,
    UnauthorizedBaseCoin,
    InsufficientAmount,
    MintRatioCannotBeZero,
}
