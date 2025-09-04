use super::AccountData;
use bytemuck::{Pod, Zeroable};
use pinocchio::{program_error::ProgramError, pubkey::Pubkey};

/// Trading statistics for a user
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct TradingStats {
    /// User wallet address
    pub user_address: Pubkey,
    /// Total trading volume in lamports
    pub total_volume: u64,
    /// Total profit/loss in lamports (can be negative)
    pub total_profit_loss: i64,
    /// Last trade timestamp
    pub last_trade_timestamp: i64,
    /// Number of trades
    pub trade_count: u32,
    /// Explicit padding to avoid implicit padding before tail
    pub _padding0: [u8; 4],
    /// Reserved space for future use
    pub reserved: [u8; 64],
}

impl AccountData for TradingStats {}

impl TradingStats {
    pub const SEED_PREFIX: &'static [u8] = b"trading_stats";

    /// Initialize new trading stats
    pub fn initialize(&mut self, user_address: Pubkey) -> Result<(), ProgramError> {
        self.user_address = user_address;
        self.total_volume = 0;
        self.total_profit_loss = 0;
        self.trade_count = 0;
        self.last_trade_timestamp = 0;
        self.reserved = [0; 64];
        Ok(())
    }

    /// Update stats after a buy trade
    pub fn update_buy(&mut self, sol_amount: u64, timestamp: i64) -> Result<(), ProgramError> {
        self.total_volume = self
            .total_volume
            .checked_add(sol_amount)
            .ok_or(ProgramError::ArithmeticOverflow)?;
        
        self.trade_count = self
            .trade_count
            .checked_add(1)
            .ok_or(ProgramError::ArithmeticOverflow)?;
        
        self.last_trade_timestamp = timestamp;
        Ok(())
    }

    /// Update stats after a sell trade
    pub fn update_sell(&mut self, sol_amount: u64, profit_loss: i64, timestamp: i64) -> Result<(), ProgramError> {
        self.total_volume = self
            .total_volume
            .checked_add(sol_amount)
            .ok_or(ProgramError::ArithmeticOverflow)?;
        
        self.total_profit_loss = self
            .total_profit_loss
            .checked_add(profit_loss)
            .ok_or(ProgramError::ArithmeticOverflow)?;
        
        self.trade_count = self
            .trade_count
            .checked_add(1)
            .ok_or(ProgramError::ArithmeticOverflow)?;
        
        self.last_trade_timestamp = timestamp;
        Ok(())
    }
}


