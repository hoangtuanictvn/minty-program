use bytemuck::{Pod, Zeroable};
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};
use pinocchio::sysvars::{clock::Clock, Sysvar};

use crate::{
    error::XTokenError,
    state::{AccountData, XToken, TradingStats},
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
    /// Treasury account (holds SOL for bonding curve)
    pub treasury: &'info AccountInfo,
    /// Fee recipient account
    pub fee_recipient: &'info AccountInfo,
    /// Buyer's trading stats account
    pub trading_stats: &'info AccountInfo,
    /// System program
    pub system_program: &'info AccountInfo,
    /// Token program
    pub token_program: &'info AccountInfo,
    /// Associated token program
    pub associated_token_program: &'info AccountInfo,
}

impl<'info> BuyTokensAccounts<'info> {
    pub fn try_from(accounts: &'info [AccountInfo]) -> Result<Self, ProgramError> {
        if accounts.len() < 10 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        Ok(Self {
            buyer: &accounts[0],
            bonding_curve: &accounts[1],
            mint: &accounts[2],
            buyer_token_account: &accounts[3],
            treasury: &accounts[4],
            fee_recipient: &accounts[5],
            trading_stats: &accounts[6],
            system_program: &accounts[7],
            token_program: &accounts[8],
            associated_token_program: &accounts[9],
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
        // Expect exactly 16 bytes: token_amount (u64 LE) + max_sol_amount (u64 LE)
        if data.len() != Self::LEN {
            return Err(ProgramError::InvalidInstructionData);
        }
        let token_amount = u64::from_le_bytes(
            data[0..8]
                .try_into()
                .map_err(|_| ProgramError::InvalidInstructionData)?,
        );
        let max_sol_amount = u64::from_le_bytes(
            data[8..16]
                .try_into()
                .map_err(|_| ProgramError::InvalidInstructionData)?,
        );
        Ok(BuyTokensInstructionData {
            token_amount,
            max_sol_amount,
        })
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

        // -------- Phase 1: Read bonding curve snapshot (immutable borrow) --------
        let (bump, _token_mint_key, total_supply_snapshot, max_supply_snapshot) = {
            let bonding_curve_data = self.accounts.bonding_curve.try_borrow_data()?;
            let bonding_curve = XToken::load(&bonding_curve_data)?;

            if bonding_curve.is_initialized == 0 {
                return Err(XTokenError::AccountNotInitialized.into());
            }

            if bonding_curve.token_mint != *self.accounts.mint.key() {
                return Err(XTokenError::InvalidAccountData.into());
            }

            // Calculate price & fee using immutable snapshot
            // (We compute below after extracting fields to minimize borrow scope if needed later.)
            (bonding_curve.bump, bonding_curve.token_mint, bonding_curve.total_supply, bonding_curve.max_supply)
        }; // immutable borrow dropped here

        // Validate supply bounds using snapshot
        let new_supply = total_supply_snapshot
            .checked_add(self.instruction_data.token_amount)
            .ok_or(ProgramError::ArithmeticOverflow)?;
        if new_supply > max_supply_snapshot {
            return Err(ProgramError::InvalidArgument);
        }

        // Re-borrow immutably to compute price and fee with helper methods
        let (total_cost, fee, sol_reserve_snapshot) = {
            let bonding_curve_data = self.accounts.bonding_curve.try_borrow_data()?;
            let bonding_curve = XToken::load(&bonding_curve_data)?;
            let total_cost = bonding_curve.calculate_buy_price(self.instruction_data.token_amount)?;
            let fee = bonding_curve.calculate_fee(total_cost)?;
            (total_cost, fee, bonding_curve.sol_reserve)
        }; // drop borrow before CPIs

        let total_with_fee = total_cost
            .checked_add(fee)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        // Check slippage protection
        if total_with_fee > self.instruction_data.max_sol_amount {
            return Err(XTokenError::SlippageExceeded.into());
        }

        // Cap treasury to 84 SOL: sol_reserve + incoming (without fee) must not exceed cap
        const SOL_CAP_LAMPORTS: u64 = 84_000_000_000; // 84 * 1e9
        let new_reserve = sol_reserve_snapshot
            .checked_add(total_cost)
            .ok_or(ProgramError::ArithmeticOverflow)?;
        if new_reserve > SOL_CAP_LAMPORTS {
            return Err(ProgramError::InvalidArgument);
        }

        // Check buyer has enough SOL
        if self.accounts.buyer.lamports() < total_with_fee {
            return Err(XTokenError::InsufficientFunds.into());
        }

        // Derive bonding curve PDA seeds
        let bump_bytes = [bump];
        let seeds = [
            pinocchio::instruction::Seed::from(XToken::SEED_PREFIX),
            pinocchio::instruction::Seed::from(self.accounts.mint.key().as_ref()),
            pinocchio::instruction::Seed::from(&bump_bytes),
        ];
        let signer = pinocchio::instruction::Signer::from(&seeds);

        // -------- Phase 2: CPI calls (no bonding_curve borrow held) --------
        // Ensure trading stats PDA exists (create if missing)
        if self.accounts.trading_stats.data_is_empty() {
            let (expected_pda, bump) = pinocchio::pubkey::find_program_address(
                &[crate::state::TradingStats::SEED_PREFIX, self.accounts.buyer.key().as_ref()],
                &crate::ID,
            );
            if expected_pda != *self.accounts.trading_stats.key() {
                return Err(ProgramError::InvalidSeeds);
            }

            let tb = [bump];
            let seeds = [
                pinocchio::instruction::Seed::from(crate::state::TradingStats::SEED_PREFIX),
                pinocchio::instruction::Seed::from(self.accounts.buyer.key().as_ref()),
                pinocchio::instruction::Seed::from(&tb),
            ];
            let signer = pinocchio::instruction::Signer::from(&seeds);

            let space = crate::state::TradingStats::LEN as u64;
            let lamports = pinocchio::sysvars::rent::Rent::get()?.minimum_balance(space as usize);

            pinocchio_system::instructions::CreateAccount {
                from: self.accounts.buyer,
                to: self.accounts.trading_stats,
                space,
                lamports,
                owner: &crate::ID,
            }
            .invoke_signed(&[signer])?;
        }
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

        // Transfer SOL from buyer to treasury (treasury holds all SOL)
        pinocchio_system::instructions::Transfer {
            from: self.accounts.buyer,
            to: self.accounts.treasury,
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

        // Mint tokens to buyer (PDA as mint authority)
        pinocchio_token::instructions::MintTo {
            mint: self.accounts.mint,
            account: self.accounts.buyer_token_account,
            mint_authority: self.accounts.bonding_curve,
            amount: self.instruction_data.token_amount,
        }
        .invoke_signed(&[signer])?;

        // -------- Phase 3: Re-borrow mutable to update state --------
        {
            let mut bonding_curve_data = self.accounts.bonding_curve.try_borrow_mut_data()?;
            let bonding_curve = XToken::load_mut(&mut bonding_curve_data)?;
            bonding_curve.update_buy(self.instruction_data.token_amount, total_cost)?;
        }

        // Update trading stats
        {
            let mut trading_stats_data = self.accounts.trading_stats.try_borrow_mut_data()?;
            let trading_stats = TradingStats::load_mut(&mut trading_stats_data)?;
            
            // Initialize if not already initialized
            if trading_stats.user_address == Pubkey::default() {
                trading_stats.initialize(*self.accounts.buyer.key())?;
            }
            
            // Get current timestamp (you might want to pass this as instruction data)
            let timestamp = Clock::get()?.unix_timestamp;
            trading_stats.update_buy(total_cost, timestamp)?;
        }

        Ok(())
    }
}
