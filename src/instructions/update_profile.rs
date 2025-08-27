use bytemuck::{Pod, Zeroable};
use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    sysvars::Sysvar,
};

use crate::{
    error::XTokenError,
    state::{AccountData, UserProfile},
};

/// Accounts for UpdateProfile instruction
pub struct UpdateProfileAccounts<'info> {
    /// User profile account (PDA)
    pub user_profile: &'info AccountInfo,
    /// User wallet (must be signer)
    pub user: &'info AccountInfo,
    /// System program
    pub system_program: &'info AccountInfo,
}

impl<'info> UpdateProfileAccounts<'info> {
    pub fn try_from(accounts: &'info [AccountInfo]) -> Result<Self, ProgramError> {
        if accounts.len() < 3 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        Ok(Self {
            user_profile: &accounts[0],
            user: &accounts[1],
            system_program: &accounts[2],
        })
    }
}

/// Instruction data for UpdateProfile
#[repr(C, packed)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct UpdateProfileInstructionData {
    /// Username length
    pub username_len: u8,
    /// Bio length
    pub bio_len: u8,
    /// Padding for alignment
    pub _padding: [u8; 2],
    /// Username (variable length, max 32 bytes)
    pub username: [u8; 32],
    /// Bio (variable length, max 200 bytes)
    pub bio: [u8; 200],
}

impl UpdateProfileInstructionData {
    pub const LEN: usize = core::mem::size_of::<UpdateProfileInstructionData>();

    pub fn get_username(&self) -> &str {
        let len = self.username_len as usize;
        if len > 32 {
            return "";
        }
        core::str::from_utf8(&self.username[..len]).unwrap_or("")
    }

    pub fn get_bio(&self) -> &str {
        let len = self.bio_len as usize;
        if len > 200 {
            return "";
        }
        core::str::from_utf8(&self.bio[..len]).unwrap_or("")
    }
}

impl<'info> TryFrom<&'info [u8]> for UpdateProfileInstructionData {
    type Error = ProgramError;

    fn try_from(data: &'info [u8]) -> Result<Self, Self::Error> {
        if data.len() != Self::LEN {
            return Err(ProgramError::InvalidInstructionData);
        }

        let result = bytemuck::try_from_bytes::<Self>(data)
            .map_err(|_| ProgramError::InvalidInstructionData)?;

        Ok(*result)
    }
}

/// UpdateProfile instruction handler
pub struct UpdateProfile<'info> {
    pub accounts: UpdateProfileAccounts<'info>,
    pub instruction_data: UpdateProfileInstructionData,
}

impl<'info> TryFrom<(&'info [AccountInfo], &'info [u8])> for UpdateProfile<'info> {
    type Error = ProgramError;

    fn try_from(
        (accounts, data): (&'info [AccountInfo], &'info [u8]),
    ) -> Result<Self, Self::Error> {
        let accounts = UpdateProfileAccounts::try_from(accounts)?;
        let instruction_data = UpdateProfileInstructionData::try_from(data)?;

        Ok(Self {
            accounts,
            instruction_data,
        })
    }
}

impl<'info> UpdateProfile<'info> {
    pub fn handler(&mut self) -> Result<(), ProgramError> {
        // Validate accounts
        if !self.accounts.user.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Validate username length
        if self.instruction_data.username_len == 0 || self.instruction_data.username_len > 32 {
            return Err(XTokenError::InvalidProfileData.into());
        }

        // Validate bio length
        if self.instruction_data.bio_len > 200 {
            return Err(XTokenError::InvalidProfileData.into());
        }

        // Derive user profile PDA
        let seeds = &[b"user_profile", self.accounts.user.key().as_ref()];
        let (user_profile_address, bump) =
            pinocchio::pubkey::find_program_address(seeds, &crate::ID);

        if user_profile_address != *self.accounts.user_profile.key() {
            return Err(ProgramError::InvalidSeeds);
        }

        // Create user profile account if it doesn't exist
        if self.accounts.user_profile.data_is_empty() {
            let space = UserProfile::LEN;
            let rent = pinocchio::sysvars::rent::Rent::get()?;
            let lamports = rent.minimum_balance(space);

            // PDA signer seeds
            let bump_bytes = [bump];
            let pda_seeds = [
                pinocchio::instruction::Seed::from(b"user_profile" as &[u8]),
                pinocchio::instruction::Seed::from(self.accounts.user.key().as_ref()),
                pinocchio::instruction::Seed::from(&bump_bytes),
            ];
            let signer = pinocchio::instruction::Signer::from(&pda_seeds);

            // Create account
            pinocchio_system::instructions::CreateAccount {
                from: self.accounts.user,
                to: self.accounts.user_profile,
                space: space as u64,
                lamports,
                owner: &crate::ID,
            }
            .invoke_signed(&[signer])?;
        }

        // Update user profile data
        let mut profile_data = self.accounts.user_profile.try_borrow_mut_data()?;
        let profile = UserProfile::load_mut(&mut profile_data)?;

        profile.update(
            *self.accounts.user.key(),
            self.instruction_data.get_username(),
            self.instruction_data.get_bio(),
        )?;

        Ok(())
    }
}
