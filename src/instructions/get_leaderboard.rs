use bytemuck::{Pod, Zeroable};
use pinocchio::{account_info::AccountInfo, program_error::ProgramError};

/// Instruction data for GetLeaderboard
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct GetLeaderboardInstructionData {
    /// Number of top traders to return (max 100)
    pub limit: u8,
    /// Offset for pagination
    pub offset: u8,
}

impl GetLeaderboardInstructionData {
    pub const LEN: usize = core::mem::size_of::<GetLeaderboardInstructionData>();
}

impl<'info> TryFrom<&'info [u8]> for GetLeaderboardInstructionData {
    type Error = ProgramError;

    fn try_from(data: &'info [u8]) -> Result<Self, Self::Error> {
        if data.len() != Self::LEN {
            return Err(ProgramError::InvalidInstructionData);
        }

        let limit = data[0];
        let offset = data[1];

        // Validate limit
        if limit == 0 || limit > 100 {
            return Err(ProgramError::InvalidArgument);
        }

        Ok(GetLeaderboardInstructionData { limit, offset })
    }
}

/// GetLeaderboard instruction handler
pub struct GetLeaderboard<'info> {
    pub accounts: &'info [AccountInfo],
    pub instruction_data: GetLeaderboardInstructionData,
}

impl<'info> TryFrom<(&'info [AccountInfo], &'info [u8])> for GetLeaderboard<'info> {
    type Error = ProgramError;

    fn try_from(
        (accounts, data): (&'info [AccountInfo], &'info [u8]),
    ) -> Result<Self, Self::Error> {
        let instruction_data = GetLeaderboardInstructionData::try_from(data)?;

        Ok(Self {
            accounts,
            instruction_data,
        })
    }
}

impl<'info> GetLeaderboard<'info> {
    pub fn handler(&self) -> Result<(), ProgramError> {
        // This instruction will return data via program logs
        // In a real implementation, you'd want to use a more efficient method
        // like returning data in the transaction logs or using a separate account

        // For now, we'll just validate the instruction
        // The actual data fetching would be done client-side by scanning accounts

        Ok(())
    }
}

/// Helper struct for leaderboard entry
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct LeaderboardEntry {
    /// User wallet address
    pub user_address: [u8; 32],
    /// Username (max 32 bytes)
    pub username: [u8; 32],
    /// Total trading volume in lamports
    pub total_volume: u64,
    /// Total profit/loss in lamports
    pub total_profit_loss: i64,
    /// Whether user is verified
    pub verified: u8,
    /// Explicit padding to align following u32 field
    pub _padding0: [u8; 3],
    /// Number of trades
    pub trade_count: u32,
    /// Reserved space (sized to make struct size a multiple of 8)
    pub reserved: [u8; 32],
}

impl LeaderboardEntry {
    pub const LEN: usize = core::mem::size_of::<LeaderboardEntry>();
}
