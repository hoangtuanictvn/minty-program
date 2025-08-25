use bytemuck::{Pod, Zeroable};
use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{
    error::XTokenError,
    state::{AccountData, XToken},
};

/// Accounts for Initialize instruction
pub struct InitializeAccounts<'info> {
    /// Authority that will control the bonding curve
    pub authority: &'info AccountInfo,
    /// Bonding curve state account (PDA)
    pub bonding_curve: &'info AccountInfo,
    /// Token mint account
    pub mint: &'info AccountInfo,
    /// Payer for account creation
    pub payer: &'info AccountInfo,
    /// System program
    pub system_program: &'info AccountInfo,
    /// Token program
    pub token_program: &'info AccountInfo,
    /// Rent sysvar
    pub rent: &'info AccountInfo,
}

impl<'info> InitializeAccounts<'info> {
    pub fn try_from(accounts: &'info [AccountInfo]) -> Result<Self, ProgramError> {
        if accounts.len() < 7 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        Ok(Self {
            authority: &accounts[0],
            bonding_curve: &accounts[1],
            mint: &accounts[2],
            payer: &accounts[3],
            system_program: &accounts[4],
            token_program: &accounts[5],
            rent: &accounts[6],
        })
    }
}

/// Instruction data for Initialize
#[repr(C, packed)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct InitializeInstructionData {
    /// Token decimals
    pub decimals: u8,
    /// Curve type (0 = linear, 1 = exponential, 2 = logarithmic)
    pub curve_type: u8,
    /// Fees in basis points (100 = 1%)
    pub fee_basis_points: u16,
    /// Padding for alignment
    pub _padding: u32,
    /// Base price in lamports per token (scaled by 1e9)
    pub base_price: u64,
    /// Slope parameter for pricing curve (scaled by 1e9)
    pub slope: u64,
    /// Maximum token supply
    pub max_supply: u64,
    /// Fee recipient
    pub fee_recipient: Pubkey,
}

impl InitializeInstructionData {
    pub const LEN: usize = core::mem::size_of::<InitializeInstructionData>();
}

impl<'info> TryFrom<&'info [u8]> for InitializeInstructionData {
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

/// Initialize instruction handler
pub struct Initialize<'info> {
    pub accounts: InitializeAccounts<'info>,
    pub instruction_data: InitializeInstructionData,
}

impl<'info> TryFrom<(&'info [AccountInfo], &'info [u8])> for Initialize<'info> {
    type Error = ProgramError;

    fn try_from(
        (accounts, data): (&'info [AccountInfo], &'info [u8]),
    ) -> Result<Self, Self::Error> {
        let accounts = InitializeAccounts::try_from(accounts)?;
        let instruction_data = InitializeInstructionData::try_from(data)?;

        Ok(Self {
            accounts,
            instruction_data,
        })
    }
}

impl<'info> Initialize<'info> {
    pub fn handler(&mut self) -> Result<(), ProgramError> {
        // Validate accounts
        if !self.accounts.authority.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if !self.accounts.payer.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Validate curve parameters
        if self.instruction_data.curve_type > 2 {
            return Err(XTokenError::InvalidCurveParameters.into());
        }

        if self.instruction_data.base_price == 0 {
            return Err(XTokenError::InvalidCurveParameters.into());
        }

        if self.instruction_data.max_supply == 0 {
            return Err(XTokenError::InvalidCurveParameters.into());
        }

        if self.instruction_data.fee_basis_points > 1000 {
            // Max 10%
            return Err(XTokenError::InvalidCurveParameters.into());
        }

        // Derive bonding curve PDA
        let seeds = &[XToken::SEED_PREFIX, self.accounts.mint.key().as_ref()];
        let (bonding_curve_address, bump) =
            pinocchio::pubkey::find_program_address(seeds, &crate::ID);

        if bonding_curve_address != *self.accounts.bonding_curve.key() {
            return Err(ProgramError::InvalidSeeds);
        }

        // Create bonding curve PDA account
        let space = XToken::LEN;
        let lamports = 1_000_000; // Minimum rent for account (simplified)

        let bump_bytes = [bump];
        let seeds = [
            Seed::from(XToken::SEED_PREFIX),
            Seed::from(self.accounts.mint.key().as_ref()),
            Seed::from(&bump_bytes),
        ];
        let signer = Signer::from(&seeds);

        pinocchio_system::instructions::CreateAccount {
            from: self.accounts.payer,
            to: self.accounts.bonding_curve,
            space: space as u64,
            lamports,
            owner: &crate::ID,
        }
        .invoke_signed(&[signer])?;

        // Verify mint account exists (should be created by client)
        if self.accounts.mint.data_is_empty() {
            return Err(ProgramError::UninitializedAccount);
        }

        // Initialize mint with bonding curve as authority
        pinocchio_token::instructions::InitializeMint2 {
            mint: self.accounts.mint,
            decimals: self.instruction_data.decimals,
            mint_authority: &bonding_curve_address,
            freeze_authority: Some(&bonding_curve_address),
        }
        .invoke()?;

        // Initialize bonding curve state
        let mut bonding_curve_data = self.accounts.bonding_curve.try_borrow_mut_data()?;
        let bonding_curve = XToken::load_mut(&mut bonding_curve_data)?;

        bonding_curve.initialize(
            *self.accounts.authority.key(),
            *self.accounts.mint.key(),
            self.instruction_data.curve_type,
            self.instruction_data.base_price,
            self.instruction_data.slope,
            self.instruction_data.max_supply,
            self.instruction_data.fee_basis_points,
            self.instruction_data.fee_recipient,
            bump,
        )?;

        Ok(())
    }
}
