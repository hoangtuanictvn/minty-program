use pinocchio::program_error::ProgramError;

pub mod initialize;
pub mod buy_tokens;
pub mod sell_tokens;

pub use initialize::*;
pub use buy_tokens::*;
pub use sell_tokens::*;

/// Instruction discriminators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Instruction {
    /// Initialize a new bonding curve
    Initialize = 0,
    /// Buy tokens from the bonding curve
    BuyTokens = 1,
    /// Sell tokens to the bonding curve
    SellTokens = 2,
}

impl TryFrom<&u8> for Instruction {
    type Error = ProgramError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Instruction::Initialize),
            1 => Ok(Instruction::BuyTokens),
            2 => Ok(Instruction::SellTokens),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}
