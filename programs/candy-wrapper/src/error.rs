use anchor_lang::prelude::*;
// An enum for custom error codes
#[error_code]
pub enum CustomError {
    IncorrectUpdateAuthority,
    IncorrectMint,
    IncorrectFeeCollector,
    UnauthorizedBaseCoin,
    InsufficientAmount,
    MintRatioCannotBeZero,
    IssuanceFeeBasisPtsCannotExceed100,
    RedemptionFeeBasisPtsCannotExceed100,
    MintIsImmutable,
    MintIsNotZero,
    BaseCoinIsNotZero,
}
