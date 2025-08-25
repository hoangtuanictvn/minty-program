use bytemuck::{Pod, Zeroable};
use pinocchio::{account_info::AccountInfo, program_error::ProgramError};

use crate::{
    error::XTokenError,
    state::{AccountData, XToken},
};

/// Accounts for SellTokens instruction
pub struct SellTokensAccounts<'info> {
    /// Seller account
    pub seller: &'info AccountInfo,
    /// Bonding curve state account
    pub bonding_curve: &'info AccountInfo,
    /// Token mint account
    pub mint: &'info AccountInfo,
    /// Seller's token account
    pub seller_token_account: &'info AccountInfo,
    /// Fee recipient account
    pub fee_recipient: &'info AccountInfo,
    /// Token program
    pub token_program: &'info AccountInfo,
}

impl<'info> SellTokensAccounts<'info> {
    pub fn try_from(accounts: &'info [AccountInfo]) -> Result<Self, ProgramError> {
        if accounts.len() < 6 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        Ok(Self {
            seller: &accounts[0],
            bonding_curve: &accounts[1],
            mint: &accounts[2],
            seller_token_account: &accounts[3],
            fee_recipient: &accounts[4],
            token_program: &accounts[5],
        })
    }
}

/// Instruction data for SellTokens
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct SellTokensInstructionData {
    /// Amount of tokens to sell
    pub token_amount: u64,
    /// Minimum SOL amount willing to accept (slippage protection)
    pub min_sol_amount: u64,
}

impl SellTokensInstructionData {
    pub const LEN: usize = core::mem::size_of::<SellTokensInstructionData>();
}

impl<'info> TryFrom<&'info [u8]> for SellTokensInstructionData {
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

/// SellTokens instruction handler
pub struct SellTokens<'info> {
    pub accounts: SellTokensAccounts<'info>,
    pub instruction_data: SellTokensInstructionData,
}

impl<'info> TryFrom<(&'info [AccountInfo], &'info [u8])> for SellTokens<'info> {
    type Error = ProgramError;

    fn try_from(
        (accounts, data): (&'info [AccountInfo], &'info [u8]),
    ) -> Result<Self, Self::Error> {
        let accounts = SellTokensAccounts::try_from(accounts)?;
        let instruction_data = SellTokensInstructionData::try_from(data)?;

        Ok(Self {
            accounts,
            instruction_data,
        })
    }
}

impl<'info> SellTokens<'info> {
    pub fn handler(&mut self) -> Result<(), ProgramError> {
        // Validate accounts
        if !self.accounts.seller.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if self.instruction_data.token_amount == 0 {
            return Err(XTokenError::InvalidTokenAmount.into());
        }

        // Load bonding curve state
        let mut bonding_curve_data = self.accounts.bonding_curve.try_borrow_mut_data()?;
        let bonding_curve = XToken::load_mut(&mut bonding_curve_data)?;

        if bonding_curve.is_initialized == 0 {
            return Err(XTokenError::AccountNotInitialized.into());
        }

        // Verify mint matches
        if bonding_curve.token_mint != *self.accounts.mint.key() {
            return Err(XTokenError::InvalidAccountData.into());
        }

        // Calculate price
        let total_proceeds =
            bonding_curve.calculate_sell_price(self.instruction_data.token_amount)?;
        let fee = bonding_curve.calculate_fee(total_proceeds)?;
        let net_proceeds = total_proceeds
            .checked_sub(fee)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        // Check slippage protection
        if net_proceeds < self.instruction_data.min_sol_amount {
            return Err(XTokenError::SlippageExceeded.into());
        }

        // Check bonding curve has enough SOL
        if self.accounts.bonding_curve.lamports() < total_proceeds {
            return Err(XTokenError::InsufficientFunds.into());
        }

        // Derive bonding curve PDA seeds
        let _bump = bonding_curve.bump;

        // Burn tokens from seller
        pinocchio_token::instructions::Burn {
            mint: self.accounts.mint,
            account: self.accounts.seller_token_account,
            authority: self.accounts.seller,
            amount: self.instruction_data.token_amount,
        }
        .invoke()?;

        // Transfer SOL from bonding curve to seller
        let mut bonding_curve_lamports = self.accounts.bonding_curve.try_borrow_mut_lamports()?;
        let mut seller_lamports = self.accounts.seller.try_borrow_mut_lamports()?;

        *bonding_curve_lamports = bonding_curve_lamports
            .checked_sub(net_proceeds)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        *seller_lamports = seller_lamports
            .checked_add(net_proceeds)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        // Transfer fee to fee recipient
        if fee > 0 {
            let mut fee_recipient_lamports =
                self.accounts.fee_recipient.try_borrow_mut_lamports()?;

            *bonding_curve_lamports = bonding_curve_lamports
                .checked_sub(fee)
                .ok_or(ProgramError::ArithmeticOverflow)?;

            *fee_recipient_lamports = fee_recipient_lamports
                .checked_add(fee)
                .ok_or(ProgramError::ArithmeticOverflow)?;
        }

        // Update bonding curve state
        bonding_curve.update_sell(self.instruction_data.token_amount, total_proceeds)?;

        Ok(())
    }
}
