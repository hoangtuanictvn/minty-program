use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
};
use pinocchio_log::log;

use crate::instructions::{Instruction, Initialize, BuyTokens, SellTokens};

/// Main instruction processor
#[inline(always)]
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> Result<(), ProgramError> {
    // Validate program ID
    if program_id != &crate::ID {
        return Err(ProgramError::IncorrectProgramId);
    }

    // Extract instruction discriminator
    let (discriminator, data) = instruction_data
        .split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;

    // Route to appropriate instruction handler
    match Instruction::try_from(discriminator)? {
        Instruction::Initialize => {
            log!("Instruction: Initialize");
            let mut initialize = Initialize::try_from((accounts, data))?;
            initialize.handler()
        }
        Instruction::BuyTokens => {
            log!("Instruction: BuyTokens");
            let mut buy_tokens = BuyTokens::try_from((accounts, data))?;
            buy_tokens.handler()
        }
        Instruction::SellTokens => {
            log!("Instruction: SellTokens");
            let mut sell_tokens = SellTokens::try_from((accounts, data))?;
            sell_tokens.handler()
        }
    }
}
