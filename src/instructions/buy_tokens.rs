use bytemuck::{Pod, Zeroable};
use pinocchio::{account_info::AccountInfo, program_error::ProgramError};

use crate::{
    error::XTokenError,
    state::{AccountData, XToken},
};

/// Accounts for BuyTokens instruction
pub struct BuyTokensAccounts<'info> {
    /// Buyer account
    pub buyer: &'info AccountInfo,
    /// Bonding curve state account
    pub bonding_curve: &'info AccountInfo,
    /// Token mint account
    pub mint: &'info AccountInfo,
    /// Buyer's token account (will be created if doesn't exist)
    pub buyer_token_account: &'info AccountInfo,
    /// Fee recipient account
    pub fee_recipient: &'info AccountInfo,
    /// System program
    pub system_program: &'info AccountInfo,
    /// Token program
    pub token_program: &'info AccountInfo,
    /// Associated token program
    pub associated_token_program: &'info AccountInfo,
}

impl<'info> BuyTokensAccounts<'info> {
    pub fn try_from(accounts: &'info [AccountInfo]) -> Result<Self, ProgramError> {
        if accounts.len() < 8 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        Ok(Self {
            buyer: &accounts[0],
            bonding_curve: &accounts[1],
            mint: &accounts[2],
            buyer_token_account: &accounts[3],
            fee_recipient: &accounts[4],
            system_program: &accounts[5],
            token_program: &accounts[6],
            associated_token_program: &accounts[7],
        })
    }
}

/// Instruction data for BuyTokens
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct BuyTokensInstructionData {
    /// Amount of tokens to buy
    pub token_amount: u64,
    /// Maximum SOL amount willing to pay (slippage protection)
    pub max_sol_amount: u64,
}

impl BuyTokensInstructionData {
    pub const LEN: usize = core::mem::size_of::<BuyTokensInstructionData>();
}

impl<'info> TryFrom<&'info [u8]> for BuyTokensInstructionData {
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

/// BuyTokens instruction handler
pub struct BuyTokens<'info> {
    pub accounts: BuyTokensAccounts<'info>,
    pub instruction_data: BuyTokensInstructionData,
}

impl<'info> TryFrom<(&'info [AccountInfo], &'info [u8])> for BuyTokens<'info> {
    type Error = ProgramError;

    fn try_from(
        (accounts, data): (&'info [AccountInfo], &'info [u8]),
    ) -> Result<Self, Self::Error> {
        let accounts = BuyTokensAccounts::try_from(accounts)?;
        let instruction_data = BuyTokensInstructionData::try_from(data)?;

        Ok(Self {
            accounts,
            instruction_data,
        })
    }
}

impl<'info> BuyTokens<'info> {
    pub fn handler(&mut self) -> Result<(), ProgramError> {
        // Validate accounts
        if !self.accounts.buyer.is_signer() {
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
        let total_cost = bonding_curve.calculate_buy_price(self.instruction_data.token_amount)?;
        let fee = bonding_curve.calculate_fee(total_cost)?;
        let total_with_fee = total_cost
            .checked_add(fee)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        // Check slippage protection
        if total_with_fee > self.instruction_data.max_sol_amount {
            return Err(XTokenError::SlippageExceeded.into());
        }

        // Check buyer has enough SOL
        if self.accounts.buyer.lamports() < total_with_fee {
            return Err(XTokenError::InsufficientFunds.into());
        }

        // Derive bonding curve PDA seeds
        // let bump = bonding_curve.bump;
        // let seeds = &[
        //     BondingCurve::SEED_PREFIX,
        //     self.accounts.mint.key().as_ref(),
        //     &[bump],
        // ];

        // Create buyer's token account if it doesn't exist
        if self.accounts.buyer_token_account.data_is_empty() {
            pinocchio_associated_token_account::instructions::Create {
                account: self.accounts.buyer_token_account,
                mint: self.accounts.mint,
                funding_account: self.accounts.buyer,
                system_program: self.accounts.system_program,
                token_program: self.accounts.token_program,
                wallet: self.accounts.buyer,
            }
            .invoke()?;
        }

        // Transfer SOL from buyer to bonding curve
        pinocchio_system::instructions::Transfer {
            from: self.accounts.buyer,
            to: self.accounts.bonding_curve,
            lamports: total_cost,
        }
        .invoke()?;

        // Transfer fee to fee recipient
        if fee > 0 {
            pinocchio_system::instructions::Transfer {
                from: self.accounts.buyer,
                to: self.accounts.fee_recipient,
                lamports: fee,
            }
            .invoke()?;
        }

        // Mint tokens to buyer
        pinocchio_token::instructions::MintTo {
            mint: self.accounts.mint,
            account: self.accounts.buyer_token_account,
            mint_authority: self.accounts.bonding_curve,
            amount: self.instruction_data.token_amount,
        }
        .invoke()?;

        // Update bonding curve state
        bonding_curve.update_buy(self.instruction_data.token_amount, total_cost)?;

        Ok(())
    }
}
