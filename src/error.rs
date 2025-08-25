use pinocchio::program_error::ProgramError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XTokenError {
    /// Invalid instruction data
    InvalidInstructionData,
    /// Invalid account data
    InvalidAccountData,
    /// Account already initialized
    AccountAlreadyInitialized,
    /// Account not initialized
    AccountNotInitialized,
    /// Insufficient funds
    InsufficientFunds,
    /// Invalid token amount
    InvalidTokenAmount,
    /// Invalid price calculation
    InvalidPriceCalculation,
    /// Slippage tolerance exceeded
    SlippageExceeded,
    /// Invalid curve parameters
    InvalidCurveParameters,
    /// Token supply exhausted
    TokenSupplyExhausted,
    /// Arithmetic overflow
    ArithmeticOverflow,
    /// Invalid authority
    InvalidAuthority,
}

impl From<XTokenError> for ProgramError {
    fn from(error: XTokenError) -> Self {
        match error {
            XTokenError::InvalidInstructionData => ProgramError::InvalidInstructionData,
            XTokenError::InvalidAccountData => ProgramError::InvalidAccountData,
            XTokenError::AccountAlreadyInitialized => ProgramError::AccountAlreadyInitialized,
            XTokenError::AccountNotInitialized => ProgramError::UninitializedAccount,
            XTokenError::InsufficientFunds => ProgramError::InsufficientFunds,
            XTokenError::InvalidTokenAmount => ProgramError::InvalidArgument,
            XTokenError::InvalidPriceCalculation => ProgramError::InvalidArgument,
            XTokenError::SlippageExceeded => ProgramError::InvalidArgument,
            XTokenError::InvalidCurveParameters => ProgramError::InvalidArgument,
            XTokenError::TokenSupplyExhausted => ProgramError::InvalidArgument,
            XTokenError::ArithmeticOverflow => ProgramError::ArithmeticOverflow,
            XTokenError::InvalidAuthority => ProgramError::InvalidArgument,
        }
    }
}
