use bytemuck::{Pod, Zeroable};
use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvars::{rent::Rent, Sysvar},
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
    /// Treasury account (PDA) - holds SOL for bonding curve
    pub treasury: &'info AccountInfo,
    /// Authority's token account (ATA) to receive pre-buy tokens
    pub authority_token_account: &'info AccountInfo,
    /// Payer for account creation
    pub payer: &'info AccountInfo,
    /// System program
    pub system_program: &'info AccountInfo,
    /// Token program
    pub token_program: &'info AccountInfo,
    /// Associated token program
    pub associated_token_program: &'info AccountInfo,
    /// Rent sysvar
    pub rent: &'info AccountInfo,
    /// Fee recipient account (for transferring initial fee)
    pub fee_recipient_account: &'info AccountInfo,
}

impl<'info> InitializeAccounts<'info> {
    pub fn try_from(accounts: &'info [AccountInfo]) -> Result<Self, ProgramError> {
        if accounts.len() < 11 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        Ok(Self {
            authority: &accounts[0],
            bonding_curve: &accounts[1],
            mint: &accounts[2],
            treasury: &accounts[3],
            authority_token_account: &accounts[4],
            payer: &accounts[5],
            system_program: &accounts[6],
            token_program: &accounts[7],
            associated_token_program: &accounts[8],
            rent: &accounts[9],
            fee_recipient_account: &accounts[10],
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
    /// Owner username (max 32 bytes) - includes length in first byte
    pub owner: [u8; 32],
    /// Base price in lamports per token (scaled by 1e9)
    pub base_price: u64,
    /// Slope parameter for pricing curve (scaled by 1e9)
    pub slope: u64,
    /// Maximum token supply
    pub max_supply: u64,
    /// Fee recipient
    pub fee_recipient: Pubkey,
    /// Optional initial pre-buy token amount (base units)
    pub initial_buy_amount: u64,
    /// Max SOL willing to pay for initial buy (slippage protection)
    pub initial_max_sol: u64,
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

        // Validate curve parameters (0=linear,1=exp,2=log,3=cpmm)
        if self.instruction_data.curve_type > 3 {
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

        // Derive treasury PDA
        let treasury_seeds = &[b"treasury", self.accounts.mint.key().as_ref()];
        let (treasury_address, treasury_bump) =
            pinocchio::pubkey::find_program_address(treasury_seeds, &crate::ID);

        if treasury_address != *self.accounts.treasury.key() {
            return Err(ProgramError::InvalidSeeds);
        }



        // Create bonding curve PDA account
        let space = XToken::LEN;
        let rent = Rent::get()?;
        let lamports = rent.minimum_balance(space);

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

        // Create treasury PDA account (system-owned, space=0)
        let treasury_space = 0; // Treasury chỉ cần space tối thiểu để hold SOL
        let treasury_lamports = rent.minimum_balance(treasury_space);

        let treasury_bump_bytes = [treasury_bump];
        let treasury_seeds = [
            Seed::from(b"treasury"),
            Seed::from(self.accounts.mint.key().as_ref()),
            Seed::from(&treasury_bump_bytes),
        ];
        let treasury_signer = Signer::from(&treasury_seeds);

        pinocchio_system::instructions::CreateAccount {
            from: self.accounts.payer,
            to: self.accounts.treasury,
            space: treasury_space as u64,
            lamports: treasury_lamports,
            owner: &pinocchio_system::ID,
        }
        .invoke_signed(&[treasury_signer])?;



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

        // Extract owner username from instruction data
        let owner_len = self.instruction_data.owner[0] as usize;
        if owner_len > 31 {
            return Err(ProgramError::InvalidArgument);
        }
        let owner_str = if owner_len == 0 {
            ""
        } else {
            core::str::from_utf8(&self.instruction_data.owner[1..=owner_len])
                .map_err(|_| ProgramError::InvalidArgument)?
        };

        bonding_curve.initialize(
            *self.accounts.authority.key(),
            *self.accounts.mint.key(),
            self.instruction_data.curve_type,
            self.instruction_data.base_price,
            self.instruction_data.slope,
            self.instruction_data.max_supply,
            self.instruction_data.fee_basis_points,
            self.instruction_data.fee_recipient,
            owner_str,
            bump,
        )?;

        // Optional: perform initial pre-buy similar to pump.fun
        if self.instruction_data.initial_buy_amount > 0 {
            // Drop mutable borrow of bonding_curve before re-borrowing
            drop(bonding_curve_data);

            // Compute cost and fee from fresh immutable snapshot
            let (total_cost, fee) = {
                let bonding_curve_data = self.accounts.bonding_curve.try_borrow_data()?;
                let bonding_curve_ro = XToken::load(&bonding_curve_data)?;
                let total_cost = bonding_curve_ro.calculate_buy_price(self.instruction_data.initial_buy_amount)?;
                let fee = bonding_curve_ro.calculate_fee(total_cost)?;
                (total_cost, fee)
            };

            let total_with_fee = total_cost
                .checked_add(fee)
                .ok_or(ProgramError::ArithmeticOverflow)?;

            // Slippage check
            if total_with_fee > self.instruction_data.initial_max_sol {
                return Err(XTokenError::SlippageExceeded.into());
            }

            // Treasury cap like in buy (84 SOL)
            const SOL_CAP_LAMPORTS: u64 = 84_000_000_000;
            let sol_reserve_snapshot = {
                let bonding_curve_data = self.accounts.bonding_curve.try_borrow_data()?;
                let bonding_curve_ro = XToken::load(&bonding_curve_data)?;
                bonding_curve_ro.sol_reserve
            };
            let new_reserve = sol_reserve_snapshot
                .checked_add(total_cost)
                .ok_or(ProgramError::ArithmeticOverflow)?;
            if new_reserve > SOL_CAP_LAMPORTS {
                return Err(ProgramError::InvalidArgument);
            }

            // Ensure payer has enough SOL
            if self.accounts.payer.lamports() < total_with_fee {
                return Err(XTokenError::InsufficientFunds.into());
            }

            // Derive signer seeds for bonding curve
            let bump_bytes = [bump];
            let seeds = [
                Seed::from(XToken::SEED_PREFIX),
                Seed::from(self.accounts.mint.key().as_ref()),
                Seed::from(&bump_bytes),
            ];
            let signer = Signer::from(&seeds);

            // Ensure ATA exists
            if self.accounts.authority_token_account.data_is_empty() {
                pinocchio_associated_token_account::instructions::Create {
                    account: self.accounts.authority_token_account,
                    mint: self.accounts.mint,
                    funding_account: self.accounts.payer,
                    system_program: self.accounts.system_program,
                    token_program: self.accounts.token_program,
                    wallet: self.accounts.authority,
                }
                .invoke()?;
            }

            // Transfer SOL to treasury and fee recipient from payer
            pinocchio_system::instructions::Transfer {
                from: self.accounts.payer,
                to: self.accounts.treasury,
                lamports: total_cost,
            }
            .invoke()?;
            if fee > 0 {
                pinocchio_system::instructions::Transfer {
                    from: self.accounts.payer,
                    to: self.accounts.fee_recipient_account,
                    lamports: fee,
                }
                .invoke()?;
            }

            // Mint tokens to authority
            pinocchio_token::instructions::MintTo {
                mint: self.accounts.mint,
                account: self.accounts.authority_token_account,
                mint_authority: self.accounts.bonding_curve,
                amount: self.instruction_data.initial_buy_amount,
            }
            .invoke_signed(&[signer])?;

            // Update state
            {
                let mut bonding_curve_data = self.accounts.bonding_curve.try_borrow_mut_data()?;
                let bonding_curve = XToken::load_mut(&mut bonding_curve_data)?;
                bonding_curve.update_buy(self.instruction_data.initial_buy_amount, total_cost)?;
            }
        }

        Ok(())
    }
}
