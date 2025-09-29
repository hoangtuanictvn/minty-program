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
    /// Owner username (max 32 bytes) - includes length in first byte
    pub owner: [u8; 32],
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
    /// Curve type (0 = linear, 3 = CPMM pump.fun-like)
    pub curve_type: u8,
    /// Whether the curve is initialized (0 = false, 1 = true)
    pub is_initialized: u8,
    /// Bump seed for PDA
    pub bump: u8,
    /// Reserved space for future use
    pub reserved: [u8; 35],
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
        owner: &str,
        bump: u8,
    ) -> Result<(), ProgramError> {
        if self.is_initialized != 0 {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        // Validate owner length (max 31 bytes for data + 1 byte for length)
        if owner.len() > 31 {
            return Err(ProgramError::InvalidArgument);
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
        self.reserved = [0; 35];

        // Store owner: first byte is length, rest is the string
        self.owner = [0; 32];
        self.owner[0] = owner.len() as u8;
        if owner.len() > 0 {
            self.owner[1..=owner.len()].copy_from_slice(owner.as_bytes());
        }

        // Set admin into reserved bytes (first 32 bytes of reserved)
        self.set_admin(fee_recipient);

        Ok(())
    }

    /// Set admin pubkey into reserved bytes [0..32]
    pub fn set_admin(&mut self, admin: Pubkey) {
        // reserved has length 35; store first 32 bytes as admin
        self.reserved[0..32].copy_from_slice(&admin);
    }

    /// Get admin pubkey from reserved bytes [0..32]
    pub fn get_admin(&self) -> Pubkey {
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&self.reserved[0..32]);
        let is_zero = bytes.iter().all(|b| *b == 0);
        if is_zero { self.fee_recipient } else { bytes }
    }

    /// Get owner username as string
    pub fn get_owner(&self) -> &str {
        let len = self.owner[0] as usize;
        if len > 31 {
            return "";
        }
        if len == 0 {
            return "";
        }
        core::str::from_utf8(&self.owner[1..=len]).unwrap_or("")
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
            3 => self.calculate_cpmm_buy(self.total_supply, new_supply),
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
            3 => self.calculate_cpmm_sell(self.total_supply, new_supply),
            _ => Err(ProgramError::InvalidArgument),
        }
    }

    /// Linear pricing: price_per_token = base_price + slope * (avg_supply_tokens)
    fn calculate_linear_price(
        &self,
        start_supply: u64,
        end_supply: u64,
    ) -> Result<u64, ProgramError> {
        let avg_supply_u128 = (start_supply as u128)
            .checked_add(end_supply as u128)
            .ok_or(ProgramError::ArithmeticOverflow)?
            .checked_div(2)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        let slope_term = (self.slope as u128)
            .checked_mul(avg_supply_u128)
            .ok_or(ProgramError::ArithmeticOverflow)?
            .checked_div(1_000_000_000u128)
            .ok_or(ProgramError::ArithmeticOverflow)?;
        let price_per_token_u128 = (self.base_price as u128)
            .checked_add(slope_term)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        let quantity_base_units_u128 = (end_supply as u128)
            .checked_sub(start_supply as u128)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        let total_u128 = price_per_token_u128
            .checked_mul(quantity_base_units_u128)
            .ok_or(ProgramError::ArithmeticOverflow)?
            .checked_div(1_000_000_000u128)
            .ok_or(ProgramError::ArithmeticOverflow)?;
        if total_u128 > u64::MAX as u128 {
            return Err(ProgramError::ArithmeticOverflow);
        }
        Ok(total_u128 as u64)
    }

    fn calculate_cpmm_buy(&self, start_supply: u64, end_supply: u64) -> Result<u64, ProgramError> {
        let x = end_supply
            .checked_sub(start_supply)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        let remaining_before = self
            .max_supply
            .checked_sub(start_supply)
            .ok_or(ProgramError::ArithmeticOverflow)?;
        let remaining_after = remaining_before
            .checked_sub(x)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        let v_s = self.base_price as u128; // lamports
        let v_t = self.slope as u128; // token base units

        let s = self.sol_reserve as u128; // lamports
        let r_before = remaining_before as u128; // tokens
        let r_after = remaining_after as u128; // tokens

        // K = (S + vS) * (R + vT)
        let k = (s.checked_add(v_s).ok_or(ProgramError::ArithmeticOverflow)?)
            .checked_mul(
                r_before
                    .checked_add(v_t)
                    .ok_or(ProgramError::ArithmeticOverflow)?,
            )
            .ok_or(ProgramError::ArithmeticOverflow)?;

        // S' = K / (R' + vT) - vS
        let denom = r_after
            .checked_add(v_t)
            .ok_or(ProgramError::ArithmeticOverflow)?;
        if denom == 0 {
            return Err(ProgramError::InvalidArgument);
        }
        let k_div = k
            .checked_div(denom)
            .ok_or(ProgramError::ArithmeticOverflow)?;
        // If k_div < v_s, S' would be negative; clamp to zero receive to avoid underflow
        if k_div <= v_s {
            return Ok(0);
        }
        let s_prime = k_div
            .checked_sub(v_s)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        // cost = S' - S
        let cost = s_prime
            .checked_sub(s)
            .ok_or(ProgramError::ArithmeticOverflow)? as u64;

        Ok(cost)
    }

    fn calculate_cpmm_sell(&self, start_supply: u64, end_supply: u64) -> Result<u64, ProgramError> {
        // amount to sell in base units (x = start - end)
        let x = start_supply
            .checked_sub(end_supply)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        // remaining tokens before/after
        let remaining_before = self
            .max_supply
            .checked_sub(start_supply)
            .ok_or(ProgramError::ArithmeticOverflow)?;
        let remaining_after = remaining_before
            .checked_add(x)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        let v_s = self.base_price as u128; // lamports (virtual)
        let v_t = self.slope as u128; // token base units (virtual)

        let s = self.sol_reserve as u128; // lamports (real)
        let r_before = remaining_before as u128; // tokens (real remaining before)
        let r_after = remaining_after as u128; // tokens (real remaining after)

        // K = (S + vS) * (R + vT)
        let k = (s.checked_add(v_s).ok_or(ProgramError::ArithmeticOverflow)?)
            .checked_mul(
                r_before
                    .checked_add(v_t)
                    .ok_or(ProgramError::ArithmeticOverflow)?,
            )
            .ok_or(ProgramError::ArithmeticOverflow)?;

        // S' = K / (R' + vT) - vS
        let denom = r_after
            .checked_add(v_t)
            .ok_or(ProgramError::ArithmeticOverflow)?;
        if denom == 0 {
            return Err(ProgramError::InvalidArgument);
        }
        let k_div = k
            .checked_div(denom)
            .ok_or(ProgramError::ArithmeticOverflow)?;
        if k_div <= v_s {
            return Ok(0);
        }
        let s_prime = k_div
            .checked_sub(v_s)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        // receive = S - S' (ensure S' <= S to avoid underflow)
        if s_prime > s {
            return Ok(0);
        }
        let receive_u128 = s
            .checked_sub(s_prime)
            .ok_or(ProgramError::ArithmeticOverflow)?;
        if receive_u128 > u64::MAX as u128 {
            return Err(ProgramError::ArithmeticOverflow);
        }
        Ok(receive_u128 as u64)
    }

    /// Calculate fees
    pub fn calculate_fee(&self, amount: u64) -> Result<u64, ProgramError> {
        // Use wider arithmetic to avoid intermediate overflow
        let amount_u128 = amount as u128;
        let bps_u128 = self.fee_basis_points as u128;
        let fee_u128 = amount_u128
            .checked_mul(bps_u128)
            .ok_or(ProgramError::ArithmeticOverflow)?
            .checked_div(10_000u128)
            .ok_or(ProgramError::ArithmeticOverflow)?;
        if fee_u128 > u64::MAX as u128 {
            return Err(ProgramError::ArithmeticOverflow);
        }
        Ok(fee_u128 as u64)
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
