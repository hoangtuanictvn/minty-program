use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
};
use pinocchio_log::log;

use crate::instructions::{Instruction, Initialize, BuyTokens, SellTokens, UpdateProfile, GetLeaderboard};

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

    // Quick validation to help debug InvalidInstructionData without dynamic logs
    // Expect first byte discriminator + fixed-size data for Initialize
    if instruction_data.is_empty() {
        log!("empty_instruction_data");
        return Err(ProgramError::InvalidInstructionData);
    }
    // accounts length validated inside instruction parser

    // Extract instruction discriminator
    let (discriminator, data) = instruction_data
        .split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;

    // Validate discriminator for Initialize (0)
    if *discriminator == 0 {
        // ok
    } else {
        log!("disc_not_zero");
    }

    // Route to appropriate instruction handler
    match Instruction::try_from(*discriminator)? {
        Instruction::Initialize => {
            log!("INIT_MARKER_V2");
            // Validate data size against Rust struct length
            if data.len() == crate::instructions::initialize::InitializeInstructionData::LEN {
                log!("initialize_data_len_ok");
            } else {
                log!("initialize_data_len_mismatch");
                return Err(ProgramError::InvalidInstructionData);
            }
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
        Instruction::UpdateProfile => {
            log!("Instruction: UpdateProfile");
            let mut update_profile = UpdateProfile::try_from((accounts, data))?;
            update_profile.handler()
        }
        Instruction::GetLeaderboard => {
            log!("Instruction: GetLeaderboard");
            let get_leaderboard = GetLeaderboard::try_from((accounts, data))?;
            get_leaderboard.handler()
        }
    }
}
