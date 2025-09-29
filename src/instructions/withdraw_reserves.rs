use bytemuck::{Pod, Zeroable};
use pinocchio::{account_info::AccountInfo, program_error::ProgramError};

use crate::{
    error::XTokenError,
    state::{AccountData, XToken},
};

/// Accounts for WithdrawReserves instruction
pub struct WithdrawReservesAccounts<'info> {
    /// Authority (must match bonding curve authority)
    pub authority: &'info AccountInfo,
    /// Bonding curve state account (PDA)
    pub bonding_curve: &'info AccountInfo,
    /// Token mint account
    pub mint: &'info AccountInfo,
    /// Treasury PDA account (system-owned)
    pub treasury: &'info AccountInfo,
    /// Recipient wallet to receive lamports
    pub recipient: &'info AccountInfo,
    /// System program
    pub system_program: &'info AccountInfo,
}

impl<'info> WithdrawReservesAccounts<'info> {
    pub fn try_from(accounts: &'info [AccountInfo]) -> Result<Self, ProgramError> {
        if accounts.len() < 6 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }
        Ok(Self {
            authority: &accounts[0],
            bonding_curve: &accounts[1],
            mint: &accounts[2],
            treasury: &accounts[3],
            recipient: &accounts[4],
            system_program: &accounts[5],
        })
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct WithdrawReservesInstructionData {
    /// 0 = withdraw all, otherwise exact lamports
    pub lamports: u64,
}

impl WithdrawReservesInstructionData {
    pub const LEN: usize = core::mem::size_of::<WithdrawReservesInstructionData>();
}

impl<'info> TryFrom<&'info [u8]> for WithdrawReservesInstructionData {
    type Error = ProgramError;
    fn try_from(data: &'info [u8]) -> Result<Self, Self::Error> {
        if data.len() != core::mem::size_of::<u64>() {
            return Err(ProgramError::InvalidInstructionData);
        }
        let lamports = u64::from_le_bytes(
            data[0..8]
                .try_into()
                .map_err(|_| ProgramError::InvalidInstructionData)?,
        );
        Ok(Self { lamports })
    }
}

pub struct WithdrawReserves<'info> {
    pub accounts: WithdrawReservesAccounts<'info>,
    pub instruction_data: WithdrawReservesInstructionData,
}

impl<'info> TryFrom<(&'info [AccountInfo], &'info [u8])> for WithdrawReserves<'info> {
    type Error = ProgramError;
    fn try_from((accounts, data): (&'info [AccountInfo], &'info [u8])) -> Result<Self, Self::Error> {
        let accounts = WithdrawReservesAccounts::try_from(accounts)?;
        let instruction_data = WithdrawReservesInstructionData::try_from(data)?;
        Ok(Self { accounts, instruction_data })
    }
}

impl<'info> WithdrawReserves<'info> {
    pub fn handler(&mut self) -> Result<(), ProgramError> {
        if !self.accounts.authority.is_signer() {
            pinocchio_log::log!("withdraw: missing authority signature");
            return Err(ProgramError::MissingRequiredSignature);
        }

        // validate state
        let bonding_curve_data = self.accounts.bonding_curve.try_borrow_data()?;
        let state = XToken::load(&bonding_curve_data)?;
        if state.is_initialized == 0 { pinocchio_log::log!("withdraw: state not initialized"); return Err(XTokenError::AccountNotInitialized.into()); }
        if state.token_mint != *self.accounts.mint.key() { pinocchio_log::log!("withdraw: mint mismatch"); return Err(XTokenError::InvalidAccountData.into()); }
        let admin = state.get_admin();
        let is_auth = state.authority == *self.accounts.authority.key() || admin == *self.accounts.authority.key();
        if !is_auth { pinocchio_log::log!("withdraw: invalid authority"); return Err(XTokenError::InvalidAuthority.into()); }

        // derive treasury PDA and signer seeds
        let (treasury_pda, treasury_bump) = pinocchio::pubkey::find_program_address(
            &[b"treasury", self.accounts.mint.key().as_ref()],
            &crate::ID,
        );
        if treasury_pda != *self.accounts.treasury.key() {
            pinocchio_log::log!("withdraw: treasury pda mismatch");
            return Err(ProgramError::InvalidSeeds);
        }

        // amount to withdraw
        let available = self.accounts.treasury.lamports();
        pinocchio_log::log!("withdraw: available={}", available);
        let amount = if self.instruction_data.lamports == 0 {
            available
        } else {
            if self.instruction_data.lamports > available { pinocchio_log::log!("withdraw: insufficient funds requested={}", self.instruction_data.lamports); return Err(XTokenError::InsufficientFunds.into()); }
            self.instruction_data.lamports
        };
        pinocchio_log::log!("withdraw: amount={}", amount);
        if amount == 0 { pinocchio_log::log!("withdraw: zero amount, skip"); return Ok(()); }

        // system-owned treasury signed transfer
        let tb = [treasury_bump];
        let seeds = [
            pinocchio::instruction::Seed::from(b"treasury"),
            pinocchio::instruction::Seed::from(self.accounts.mint.key().as_ref()),
            pinocchio::instruction::Seed::from(&tb),
        ];
        let signer = pinocchio::instruction::Signer::from(&seeds);

        pinocchio_system::instructions::Transfer {
            from: self.accounts.treasury,
            to: self.accounts.recipient,
            lamports: amount,
        }.invoke_signed(&[signer])?;

        Ok(())
    }
}


