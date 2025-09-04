use bytemuck::{Pod, Zeroable};
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};
use pinocchio::sysvars::{clock::Clock, Sysvar};

use crate::{
    error::XTokenError,
    state::{AccountData, XToken, TradingStats},
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
    /// Treasury account (holds SOL for bonding curve)
    pub treasury: &'info AccountInfo,
    /// Fee recipient account
    pub fee_recipient: &'info AccountInfo,
    /// Seller's trading stats account
    pub trading_stats: &'info AccountInfo,
    /// Token program
    pub token_program: &'info AccountInfo,
    /// System program
    pub system_program: &'info AccountInfo,
}

impl<'info> SellTokensAccounts<'info> {
    pub fn try_from(accounts: &'info [AccountInfo]) -> Result<Self, ProgramError> {
        if accounts.len() < 9 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        Ok(Self {
            seller: &accounts[0],
            bonding_curve: &accounts[1],
            mint: &accounts[2],
            seller_token_account: &accounts[3],
            treasury: &accounts[4],
            fee_recipient: &accounts[5],
            trading_stats: &accounts[6],
            token_program: &accounts[7],
            system_program: &accounts[8],
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
        // Expect exactly 16 bytes: token_amount (u64 LE) + min_sol_amount (u64 LE)
        if data.len() != 16 {
            return Err(ProgramError::InvalidInstructionData);
        }

        let token_amount = u64::from_le_bytes(
            data[0..8]
                .try_into()
                .map_err(|_| ProgramError::InvalidInstructionData)?,
        );
        let min_sol_amount = u64::from_le_bytes(
            data[8..16]
                .try_into()
                .map_err(|_| ProgramError::InvalidInstructionData)?,
        );

        Ok(SellTokensInstructionData {
            token_amount,
            min_sol_amount,
        })
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

        // -------- Phase 1: Read bonding curve snapshot (immutable borrow) --------
        let (bump, _token_mint_key, _total_supply_snapshot, total_proceeds, fee, net_proceeds) = {
            let bonding_curve_data = self.accounts.bonding_curve.try_borrow_data()?;
            let bonding_curve = XToken::load(&bonding_curve_data)?;

            if bonding_curve.is_initialized == 0 {
                return Err(XTokenError::AccountNotInitialized.into());
            }

            // Verify mint matches
            if bonding_curve.token_mint != *self.accounts.mint.key() {
                return Err(XTokenError::InvalidAccountData.into());
            }

            // Calculate price and fee using immutable snapshot
            let total_proceeds = bonding_curve.calculate_sell_price(self.instruction_data.token_amount)?;
            let fee = bonding_curve.calculate_fee(total_proceeds)?;
            let net_proceeds = total_proceeds
                .checked_sub(fee)
                .ok_or(ProgramError::ArithmeticOverflow)?;

            (bonding_curve.bump, bonding_curve.token_mint, bonding_curve.total_supply, total_proceeds, fee, net_proceeds)
        }; // immutable borrow dropped here

        // Check slippage protection
        if net_proceeds < self.instruction_data.min_sol_amount {
            return Err(XTokenError::SlippageExceeded.into());
        }

        // Check treasury has enough SOL
        if self.accounts.treasury.lamports() < total_proceeds {
            return Err(XTokenError::InsufficientFunds.into());
        }

        // Derive bonding curve PDA seeds (for potential mint auth usage) and treasury seeds for signed transfers
        let bump_bytes = [bump];
        let bc_seeds = [
            pinocchio::instruction::Seed::from(XToken::SEED_PREFIX),
            pinocchio::instruction::Seed::from(self.accounts.mint.key().as_ref()),
            pinocchio::instruction::Seed::from(&bump_bytes),
        ];
        let _bonding_curve_signer = pinocchio::instruction::Signer::from(&bc_seeds);

        // -------- Phase 2: CPI calls (no bonding_curve borrow held) --------
        // Ensure trading stats PDA exists (create if missing)
        if self.accounts.trading_stats.data_is_empty() {
            let (expected_pda, bump) = pinocchio::pubkey::find_program_address(
                &[crate::state::TradingStats::SEED_PREFIX, self.accounts.seller.key().as_ref()],
                &crate::ID,
            );
            if expected_pda != *self.accounts.trading_stats.key() {
                return Err(ProgramError::InvalidSeeds);
            }

            let tb = [bump];
            let seeds = [
                pinocchio::instruction::Seed::from(crate::state::TradingStats::SEED_PREFIX),
                pinocchio::instruction::Seed::from(self.accounts.seller.key().as_ref()),
                pinocchio::instruction::Seed::from(&tb),
            ];
            let signer = pinocchio::instruction::Signer::from(&seeds);

            let space = crate::state::TradingStats::LEN as u64;
            let lamports = pinocchio::sysvars::rent::Rent::get()?.minimum_balance(space as usize);

            pinocchio_system::instructions::CreateAccount {
                from: self.accounts.seller,
                to: self.accounts.trading_stats,
                space,
                lamports,
                owner: &crate::ID,
            }
            .invoke_signed(&[signer])?;
        }
        // Burn tokens from seller
        pinocchio_token::instructions::Burn {
            mint: self.accounts.mint,
            account: self.accounts.seller_token_account,
            authority: self.accounts.seller,
            amount: self.instruction_data.token_amount,
        }
        .invoke()?;



        // Transfer SOL from treasury to seller/fee
        // Support both treasury owner patterns:
        // - System Program owned PDA (space=0): use invoke_signed(SystemProgram::Transfer)
        // - Program owned account with data: mutate lamports directly
        let is_system_owned_treasury = unsafe { *self.accounts.treasury.owner() == pinocchio_system::ID };
        if is_system_owned_treasury {
            // System-owned treasury: signed transfers
            let (treasury_pda, treasury_bump) = pinocchio::pubkey::find_program_address(
                &[b"treasury", self.accounts.mint.key().as_ref()],
                &crate::ID,
            );

            if treasury_pda != *self.accounts.treasury.key() {
                return Err(ProgramError::InvalidSeeds);
            }

            let tb_bytes = [treasury_bump];
            let treasury_seeds = [
                pinocchio::instruction::Seed::from(b"treasury"),
                pinocchio::instruction::Seed::from(self.accounts.mint.key().as_ref()),
                pinocchio::instruction::Seed::from(&tb_bytes),
            ];
            let treasury_signer = pinocchio::instruction::Signer::from(&treasury_seeds);

            pinocchio_system::instructions::Transfer {
                from: self.accounts.treasury,
                to: self.accounts.seller,
                lamports: net_proceeds,
            }
            .invoke_signed(&[treasury_signer])?;

            if fee > 0 {
                // Recreate signer since previous invoke_signed moved it
                let tb_bytes2 = [treasury_bump];
                let treasury_seeds2 = [
                    pinocchio::instruction::Seed::from(b"treasury"),
                    pinocchio::instruction::Seed::from(self.accounts.mint.key().as_ref()),
                    pinocchio::instruction::Seed::from(&tb_bytes2),
                ];
                let treasury_signer2 = pinocchio::instruction::Signer::from(&treasury_seeds2);

                pinocchio_system::instructions::Transfer {
                    from: self.accounts.treasury,
                    to: self.accounts.fee_recipient,
                    lamports: fee,
                }
                .invoke_signed(&[treasury_signer2])?;
            }
        } else {
            // Program-owned treasury: mutate lamports directly
            {
                let mut treasury_lamports = self.accounts.treasury.try_borrow_mut_lamports()?;
                let mut seller_lamports = self.accounts.seller.try_borrow_mut_lamports()?;
                // safety checks
                if *treasury_lamports < net_proceeds {
                    return Err(XTokenError::InsufficientFunds.into());
                }
                *treasury_lamports = treasury_lamports
                    .checked_sub(net_proceeds)
                    .ok_or(ProgramError::ArithmeticOverflow)?;
                *seller_lamports = seller_lamports
                    .checked_add(net_proceeds)
                    .ok_or(ProgramError::ArithmeticOverflow)?;
            }
            if fee > 0 {
                let mut treasury_lamports = self.accounts.treasury.try_borrow_mut_lamports()?;
                let mut fee_lamports = self.accounts.fee_recipient.try_borrow_mut_lamports()?;
                if *treasury_lamports < fee {
                    return Err(XTokenError::InsufficientFunds.into());
                }
                *treasury_lamports = treasury_lamports
                    .checked_sub(fee)
                    .ok_or(ProgramError::ArithmeticOverflow)?;
                *fee_lamports = fee_lamports
                    .checked_add(fee)
                    .ok_or(ProgramError::ArithmeticOverflow)?;
            }
        }

        // -------- Phase 3: Re-borrow mutable to update state --------
        {
            let mut bonding_curve_data = self.accounts.bonding_curve.try_borrow_mut_data()?;
            let bonding_curve = XToken::load_mut(&mut bonding_curve_data)?;
            bonding_curve.update_sell(self.instruction_data.token_amount, total_proceeds)?;
        }

        // Update trading stats (temporarily disable P&L accumulation)
        {
            let mut trading_stats_data = self.accounts.trading_stats.try_borrow_mut_data()?;
            let trading_stats = TradingStats::load_mut(&mut trading_stats_data)?;
            
            // Initialize if not already initialized
            if trading_stats.user_address == Pubkey::default() {
                trading_stats.initialize(*self.accounts.seller.key())?;
            }
            
            // Do not accumulate profit/loss for now
            let profit_loss: i64 = 0;
            
            // Get current timestamp
            let timestamp = Clock::get()?.unix_timestamp;
            trading_stats.update_sell(total_proceeds, profit_loss, timestamp)?;
        }

        Ok(())
    }
}
