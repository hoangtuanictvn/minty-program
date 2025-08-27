use super::AccountData;
use bytemuck::{Pod, Zeroable};
use pinocchio::{program_error::ProgramError, pubkey::Pubkey};

/// User profile state account
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct UserProfile {
    /// User wallet address
    pub user_address: Pubkey,
    /// Username length
    pub username_len: u8,
    /// Bio length
    pub bio_len: u8,
    /// Padding for alignment
    pub _padding: [u8; 2],
    /// Username (max 32 bytes)
    pub username: [u8; 32],
    /// Bio (max 200 bytes)
    pub bio: [u8; 200],
    /// Whether the profile is initialized (0 = false, 1 = true)
    pub is_initialized: u8,
    /// Reserved space for future use
    pub reserved: [u8; 64],
}

impl AccountData for UserProfile {}

impl UserProfile {
    pub const SEED_PREFIX: &'static [u8] = b"user_profile";

    /// Update user profile
    pub fn update(
        &mut self,
        user_address: Pubkey,
        username: &str,
        bio: &str,
    ) -> Result<(), ProgramError> {
        // Validate username length
        if username.len() > 32 {
            return Err(ProgramError::InvalidArgument);
        }

        // Validate bio length
        if bio.len() > 200 {
            return Err(ProgramError::InvalidArgument);
        }

        // Update profile data
        self.user_address = user_address;
        self.username_len = username.len() as u8;
        self.bio_len = bio.len() as u8;
        self.is_initialized = 1; // true

        // Copy username
        self.username = [0; 32];
        self.username[..username.len()].copy_from_slice(username.as_bytes());

        // Copy bio
        self.bio = [0; 200];
        self.bio[..bio.len()].copy_from_slice(bio.as_bytes());

        Ok(())
    }

    /// Get username as string
    pub fn get_username(&self) -> &str {
        let len = self.username_len as usize;
        if len > 32 {
            return "";
        }
        core::str::from_utf8(&self.username[..len]).unwrap_or("")
    }

    /// Get bio as string
    pub fn get_bio(&self) -> &str {
        let len = self.bio_len as usize;
        if len > 200 {
            return "";
        }
        core::str::from_utf8(&self.bio[..len]).unwrap_or("")
    }
}
