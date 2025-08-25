use super::AccountData;
use bytemuck::{Pod, Zeroable};
use pinocchio::{program_error::ProgramError, pubkey::Pubkey};

/// Bonding curve state account
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct XToken {
    /// Authority that can update curve parameters
    pub authority: Pubkey,
    /// Token mint address
    pub token_mint: Pubkey,
    /// Fee recipient
    pub fee_recipient: Pubkey,
    /// SOL reserve (in lamports)
    pub sol_reserve: u64,
    /// Token reserve (in token units)
    pub token_reserve: u64,
    /// Total token supply
    pub total_supply: u64,
    /// Base price in lamports per token (scaled by 1e9)
    pub base_price: u64,
    /// Slope parameter for pricing curve (scaled by 1e9)
    pub slope: u64,
    /// Maximum token supply
    pub max_supply: u64,
    /// Fees in basis points (100 = 1%)
    pub fee_basis_points: u16,
    /// Curve type (0 = linear, 1 = exponential, 2 = logarithmic)
    pub curve_type: u8,
    /// Whether the curve is initialized (0 = false, 1 = true)
    pub is_initialized: u8,
    /// Bump seed for PDA
    pub bump: u8,
    /// Padding for alignment
    pub _padding: [u8; 3],
    /// Reserved space for future use
    pub reserved: [u8; 64],
}

impl AccountData for XToken {}

impl XToken {
    pub const SEED_PREFIX: &'static [u8] = b"x_token";

    /// Initialize a new bonding curve
    pub fn initialize(
        &mut self,
        authority: Pubkey,
        token_mint: Pubkey,
        curve_type: u8,
        base_price: u64,
        slope: u64,
        max_supply: u64,
        fee_basis_points: u16,
        fee_recipient: Pubkey,
        bump: u8,
    ) -> Result<(), ProgramError> {
        if self.is_initialized != 0 {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        self.authority = authority;
        self.token_mint = token_mint;
        self.sol_reserve = 0;
        self.token_reserve = 0;
        self.total_supply = 0;
        self.curve_type = curve_type;
        self.base_price = base_price;
        self.slope = slope;
        self.max_supply = max_supply;
        self.fee_basis_points = fee_basis_points;
        self.fee_recipient = fee_recipient;
        self.is_initialized = 1; // true
        self.bump = bump;
        self.reserved = [0; 64];

        Ok(())
    }

    /// Calculate price for buying tokens
    pub fn calculate_buy_price(&self, token_amount: u64) -> Result<u64, ProgramError> {
        if token_amount == 0 {
            return Ok(0);
        }

        let new_supply = self
            .total_supply
            .checked_add(token_amount)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        if new_supply > self.max_supply {
            return Err(ProgramError::InvalidArgument);
        }

        match self.curve_type {
            0 => self.calculate_linear_price(self.total_supply, new_supply),
            1 => self.calculate_exponential_price(self.total_supply, new_supply),
            2 => self.calculate_logarithmic_price(self.total_supply, new_supply),
            _ => Err(ProgramError::InvalidArgument),
        }
    }

    /// Calculate price for selling tokens
    pub fn calculate_sell_price(&self, token_amount: u64) -> Result<u64, ProgramError> {
        if token_amount == 0 {
            return Ok(0);
        }

        if token_amount > self.total_supply {
            return Err(ProgramError::InvalidArgument);
        }

        let new_supply = self
            .total_supply
            .checked_sub(token_amount)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        match self.curve_type {
            0 => self.calculate_linear_price(new_supply, self.total_supply),
            1 => self.calculate_exponential_price(new_supply, self.total_supply),
            2 => self.calculate_logarithmic_price(new_supply, self.total_supply),
            _ => Err(ProgramError::InvalidArgument),
        }
    }

    /// Linear pricing: price = base_price + slope * supply
    fn calculate_linear_price(
        &self,
        start_supply: u64,
        end_supply: u64,
    ) -> Result<u64, ProgramError> {
        let avg_supply = start_supply
            .checked_add(end_supply)
            .ok_or(ProgramError::ArithmeticOverflow)?
            .checked_div(2)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        let price_per_token = self
            .base_price
            .checked_add(
                self.slope
                    .checked_mul(avg_supply)
                    .ok_or(ProgramError::ArithmeticOverflow)?
                    .checked_div(1_000_000_000) // Scale down
                    .ok_or(ProgramError::ArithmeticOverflow)?,
            )
            .ok_or(ProgramError::ArithmeticOverflow)?;

        let quantity = end_supply
            .checked_sub(start_supply)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        price_per_token
            .checked_mul(quantity)
            .ok_or(ProgramError::ArithmeticOverflow.into())
    }

    /// Exponential pricing: price = base_price * (1 + slope)^supply
    fn calculate_exponential_price(
        &self,
        start_supply: u64,
        end_supply: u64,
    ) -> Result<u64, ProgramError> {
        // Simplified exponential calculation for demonstration
        // In production, you'd want more sophisticated math
        let quantity = end_supply
            .checked_sub(start_supply)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        let multiplier = 1_000_000_000u64
            .checked_add(self.slope)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        let price_per_token = self
            .base_price
            .checked_mul(multiplier)
            .ok_or(ProgramError::ArithmeticOverflow)?
            .checked_div(1_000_000_000)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        price_per_token
            .checked_mul(quantity)
            .ok_or(ProgramError::ArithmeticOverflow.into())
    }

    /// Logarithmic pricing: price = base_price * log(1 + slope * supply)
    fn calculate_logarithmic_price(
        &self,
        start_supply: u64,
        end_supply: u64,
    ) -> Result<u64, ProgramError> {
        // Simplified logarithmic calculation for demonstration
        let quantity = end_supply
            .checked_sub(start_supply)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        let avg_supply = start_supply
            .checked_add(end_supply)
            .ok_or(ProgramError::ArithmeticOverflow)?
            .checked_div(2)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        // Simple approximation: log effect reduces price growth
        let log_factor = 1_000_000_000u64
            .checked_add(avg_supply.checked_div(1000).unwrap_or(0))
            .ok_or(ProgramError::ArithmeticOverflow)?;

        let price_per_token = self
            .base_price
            .checked_mul(log_factor)
            .ok_or(ProgramError::ArithmeticOverflow)?
            .checked_div(1_000_000_000)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        price_per_token
            .checked_mul(quantity)
            .ok_or(ProgramError::ArithmeticOverflow.into())
    }

    /// Calculate fees
    pub fn calculate_fee(&self, amount: u64) -> Result<u64, ProgramError> {
        amount
            .checked_mul(self.fee_basis_points as u64)
            .ok_or(ProgramError::ArithmeticOverflow)?
            .checked_div(10_000)
            .ok_or(ProgramError::ArithmeticOverflow.into())
    }

    /// Update reserves after buy
    pub fn update_buy(&mut self, token_amount: u64, sol_amount: u64) -> Result<(), ProgramError> {
        self.total_supply = self
            .total_supply
            .checked_add(token_amount)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        self.token_reserve = self
            .token_reserve
            .checked_add(token_amount)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        self.sol_reserve = self
            .sol_reserve
            .checked_add(sol_amount)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        Ok(())
    }

    /// Update reserves after sell
    pub fn update_sell(&mut self, token_amount: u64, sol_amount: u64) -> Result<(), ProgramError> {
        self.total_supply = self
            .total_supply
            .checked_sub(token_amount)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        self.token_reserve = self
            .token_reserve
            .checked_sub(token_amount)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        self.sol_reserve = self
            .sol_reserve
            .checked_sub(sol_amount)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        Ok(())
    }
}
