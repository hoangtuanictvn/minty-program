use bytemuck::{Pod, Zeroable};
use pinocchio::program_error::ProgramError;

pub mod x_token;

pub use x_token::*;

/// Trait for loading and storing account data
pub trait AccountData: Pod + Zeroable {
    const LEN: usize = core::mem::size_of::<Self>();

    fn load(data: &[u8]) -> Result<&Self, ProgramError> {
        if data.len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(bytemuck::from_bytes(data))
    }

    fn load_mut(data: &mut [u8]) -> Result<&mut Self, ProgramError> {
        if data.len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(bytemuck::from_bytes_mut(data))
    }
}
