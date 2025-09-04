use pinocchio::program_error::ProgramError;

pub mod initialize;
pub mod buy_tokens;
pub mod sell_tokens;
pub mod update_profile;
pub mod get_leaderboard;

// Re-export structs for processor to use
pub use initialize::Initialize;
pub use buy_tokens::BuyTokens;
pub use sell_tokens::SellTokens;
pub use update_profile::UpdateProfile;
pub use get_leaderboard::GetLeaderboard;

#[derive(Debug)]
pub enum Instruction {
    Initialize,
    BuyTokens,
    SellTokens,
    UpdateProfile,
    GetLeaderboard,
}

impl TryFrom<u8> for Instruction {
    type Error = ProgramError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Instruction::Initialize),
            1 => Ok(Instruction::BuyTokens),
            2 => Ok(Instruction::SellTokens),
            3 => Ok(Instruction::UpdateProfile),
            4 => Ok(Instruction::GetLeaderboard),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}
