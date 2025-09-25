use pinocchio::program_error::ProgramError;

pub mod buy_tokens;
pub mod initialize;
pub mod sell_tokens;

// Re-export structs for processor to use
pub use buy_tokens::BuyTokens;
pub use initialize::Initialize;
pub use sell_tokens::SellTokens;

#[derive(Debug)]
pub enum Instruction {
    Initialize,
    BuyTokens,
    SellTokens,
}

impl TryFrom<u8> for Instruction {
    type Error = ProgramError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Instruction::Initialize),
            1 => Ok(Instruction::BuyTokens),
            2 => Ok(Instruction::SellTokens),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}
