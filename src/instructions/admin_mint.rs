use bytemuck::{Pod, Zeroable};
use pinocchio::{account_info::AccountInfo, program_error::ProgramError};

use crate::{
    error::XTokenError,
    state::{AccountData, XToken},
};

/// Accounts for AdminMint instruction
pub struct AdminMintAccounts<'info> {
    /// Authority (must match bonding curve authority)
    pub authority: &'info AccountInfo,
    /// Bonding curve state account (PDA)
    pub bonding_curve: &'info AccountInfo,
    /// Token mint account
    pub mint: &'info AccountInfo,
    /// Recipient token account (ATA)
    pub recipient_token_account: &'info AccountInfo,
    /// Token program
    pub token_program: &'info AccountInfo,
}

impl<'info> AdminMintAccounts<'info> {
    pub fn try_from(accounts: &'info [AccountInfo]) -> Result<Self, ProgramError> {
        if accounts.len() < 5 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }
        Ok(Self {
            authority: &accounts[0],
            bonding_curve: &accounts[1],
            mint: &accounts[2],
            recipient_token_account: &accounts[3],
            token_program: &accounts[4],
        })
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct AdminMintInstructionData {
    /// Amount to mint (base units)
    pub amount: u64,
}

impl AdminMintInstructionData {
    pub const LEN: usize = core::mem::size_of::<AdminMintInstructionData>();
}

impl<'info> TryFrom<&'info [u8]> for AdminMintInstructionData {
    type Error = ProgramError;
    fn try_from(data: &'info [u8]) -> Result<Self, Self::Error> {
        if data.len() != core::mem::size_of::<u64>() {
            return Err(ProgramError::InvalidInstructionData);
        }
        let amount = u64::from_le_bytes(
            data[0..8]
                .try_into()
                .map_err(|_| ProgramError::InvalidInstructionData)?,
        );
        Ok(Self { amount })
    }
}

pub struct AdminMint<'info> {
    pub accounts: AdminMintAccounts<'info>,
    pub instruction_data: AdminMintInstructionData,
}

impl<'info> TryFrom<(&'info [AccountInfo], &'info [u8])> for AdminMint<'info> {
    type Error = ProgramError;
    fn try_from((accounts, data): (&'info [AccountInfo], &'info [u8])) -> Result<Self, Self::Error> {
        let accounts = AdminMintAccounts::try_from(accounts)?;
        let instruction_data = AdminMintInstructionData::try_from(data)?;
        Ok(Self { accounts, instruction_data })
    }
}

impl<'info> AdminMint<'info> {
    pub fn handler(&mut self) -> Result<(), ProgramError> {
        if !self.accounts.authority.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if self.instruction_data.amount == 0 {
            return Err(XTokenError::InvalidTokenAmount.into());
        }

        // validate state
        let bonding_curve_data = self.accounts.bonding_curve.try_borrow_data()?;
        let state = XToken::load(&bonding_curve_data)?;
        if state.is_initialized == 0 { return Err(XTokenError::AccountNotInitialized.into()); }
        if state.token_mint != *self.accounts.mint.key() { return Err(XTokenError::InvalidAccountData.into()); }
        let admin = state.get_admin();
        let is_auth = state.authority == *self.accounts.authority.key() || admin == *self.accounts.authority.key();
        if !is_auth { return Err(XTokenError::InvalidAuthority.into()); }

        // derive bonding curve PDA and signer seeds
        let (bonding_curve_pda, bonding_curve_bump) = pinocchio::pubkey::find_program_address(
            &[XToken::SEED_PREFIX, self.accounts.mint.key().as_ref()],
            &crate::ID,
        );
        if bonding_curve_pda != *self.accounts.bonding_curve.key() {
            return Err(ProgramError::InvalidSeeds);
        }

        // mint tokens using bonding curve as mint authority
        let bump_bytes = [bonding_curve_bump];
        let seeds = [
            pinocchio::instruction::Seed::from(XToken::SEED_PREFIX),
            pinocchio::instruction::Seed::from(self.accounts.mint.key().as_ref()),
            pinocchio::instruction::Seed::from(&bump_bytes),
        ];
        let signer = pinocchio::instruction::Signer::from(&seeds);

        pinocchio_token::instructions::MintTo {
            mint: self.accounts.mint,
            account: self.accounts.recipient_token_account,
            mint_authority: self.accounts.bonding_curve,
            amount: self.instruction_data.amount,
        }
        .invoke_signed(&[signer])?;

        Ok(())
    }
}
