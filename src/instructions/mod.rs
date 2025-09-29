use pinocchio::program_error::ProgramError;

pub mod initialize;
pub mod buy_tokens;
pub mod sell_tokens;
pub mod withdraw_reserves;
pub mod admin_mint;

// Re-export structs for processor to use
pub use initialize::Initialize;
pub use buy_tokens::BuyTokens;
pub use sell_tokens::SellTokens;
pub use withdraw_reserves::WithdrawReserves;
pub use admin_mint::AdminMint;

#[derive(Debug)]
pub enum Instruction {
    Initialize,
    BuyTokens,
    SellTokens,
    WithdrawReserves,
    AdminMint,
}

impl TryFrom<u8> for Instruction {
    type Error = ProgramError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Instruction::Initialize),
            1 => Ok(Instruction::BuyTokens),
            2 => Ok(Instruction::SellTokens),
            3 => Ok(Instruction::WithdrawReserves),
            4 => Ok(Instruction::AdminMint),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}
