use bytemuck::{Pod, Zeroable};
use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Seed, Signer},
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvars::{rent::Rent, Sysvar},
};

// No heap allocations in SBF

use crate::{
    error::XTokenError,
    state::{AccountData, XToken},
};

// Metaplex Token Metadata Program ID: metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s
pub const METAPLEX_TOKEN_METADATA_ID: [u8; 32] = [ 11, 112, 101, 177, 227, 209, 124, 69, 56, 157, 82, 127, 107, 4, 195, 205, 88, 184, 108, 115, 26, 160, 253, 181, 73, 182, 209, 188, 3, 248, 41, 70, ];

// Metadata prefix for PDA derivation
pub const METADATA_PREFIX: &[u8] = b"metadata";

// Metaplex instruction discriminator
pub const CREATE_METADATA_ACCOUNT_V3: u8 = 33;

// Rent sysvar program ID
pub const RENT_SYSVAR_ID: [u8; 32] = [
    0x06, 0xa7, 0xd5, 0x17, 0x18, 0x7b, 0xd1, 0x6e, 0x03, 0x5b, 0xbd, 0x6b, 0xde, 0x32, 0x6b, 0x31,
    0x4f, 0x6e, 0x57, 0x5c, 0x0e, 0x68, 0xd1, 0x51, 0x9e, 0x5d, 0x9e, 0x5d, 0x38, 0xf0, 0x8e, 0x06
];

/// Accounts for Initialize instruction with metadata support
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
    /// Metadata account (PDA from Metaplex)
    pub metadata_account: &'info AccountInfo,
    /// Metaplex Token Metadata Program
    pub metaplex_program: &'info AccountInfo,
}

impl<'info> InitializeAccounts<'info> {
    pub fn try_from(accounts: &'info [AccountInfo]) -> Result<Self, ProgramError> {
        if accounts.len() < 13 {
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
            metadata_account: &accounts[11],
            metaplex_program: &accounts[12],
        })
    }
}

/// Instruction data for Initialize with metadata
#[repr(C, packed)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct InitializeInstructionData {
    /// Token decimals
    pub decimals: u8,
    /// Curve type (0 = linear, 1 = exponential, 2 = logarithmic, 3 = cpmm)
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
    /// Token name (max 32 bytes) - includes length in first byte
    pub token_name: [u8; 32],
    /// Token symbol (max 10 bytes) - includes length in first byte
    pub token_symbol: [u8; 10],
    /// Token metadata URI (max 200 bytes) - includes length in first byte
    pub token_uri: [u8; 200],
}

impl InitializeInstructionData {
    pub const LEN: usize = core::mem::size_of::<InitializeInstructionData>();
    
    /// Extract &str from fixed-size array with length prefix (no alloc)
    fn extract_str<'a>(data: &'a [u8]) -> Result<&'a str, ProgramError> {
        if data.is_empty() {
            return Ok("");
        }
        
        let len = data[0] as usize;
        if len == 0 {
            return Ok("");
        }
        
        // Safe bounds check
        if len >= data.len() || len > data.len() - 1 {
            pinocchio::msg!("Invalid string length for data");
            return Err(ProgramError::InvalidInstructionData);
        }
        
        // Additional safety check
        let end_idx = 1 + len;
        if end_idx > data.len() {
            pinocchio::msg!("String end index exceeds data length");
            return Err(ProgramError::InvalidInstructionData);
        }
        
        core::str::from_utf8(&data[1..end_idx]).map_err(|_| ProgramError::InvalidInstructionData)
    }
    
    pub fn get_token_name(&self) -> Result<&str, ProgramError> {
        Self::extract_str(&self.token_name)
    }
    
    pub fn get_token_symbol(&self) -> Result<&str, ProgramError> {
        Self::extract_str(&self.token_symbol)
    }
    
    pub fn get_token_uri(&self) -> Result<&str, ProgramError> {
        Self::extract_str(&self.token_uri)
    }
    
    pub fn get_owner(&self) -> Result<&str, ProgramError> {
        Self::extract_str(&self.owner)
    }
}

impl<'info> TryFrom<&'info [u8]> for InitializeInstructionData {
    type Error = ProgramError;

    fn try_from(data: &'info [u8]) -> Result<Self, Self::Error> {
        pinocchio::msg!("Starting InitializeInstructionData::try_from");
        
        let expected_len = Self::LEN;
        let actual_len = data.len();
        
        if actual_len != expected_len {
            pinocchio::msg!("Invalid instruction data length - size mismatch");
            return Err(ProgramError::InvalidInstructionData);
        }
        pinocchio::msg!("Data length validation passed");

        let result = bytemuck::try_from_bytes::<Self>(data)
            .map_err(|_| {
                pinocchio::msg!("Bytemuck deserialization failed");
                ProgramError::InvalidInstructionData
            })?;

        pinocchio::msg!("InitializeInstructionData created successfully");
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
        pinocchio::msg!("Starting Initialize::try_from");
        
        let accounts = InitializeAccounts::try_from(accounts)?;
        pinocchio::msg!("Accounts parsed successfully");
        
        let instruction_data = InitializeInstructionData::try_from(data)?;
        pinocchio::msg!("Instruction data parsed successfully");

        Ok(Self {
            accounts,
            instruction_data,
        })
    }
}

/// Fixed-size metadata serialization to avoid heap allocation
fn serialize_metadata_v2_fixed(buf: &mut [u8], offset: &mut usize, name: &str, symbol: &str, uri: &str) -> Result<(), ProgramError> {
    let write = |b: &mut [u8], o: &mut usize, s: &[u8]| -> Result<(), ProgramError> {
        if *o + s.len() > b.len() { return Err(ProgramError::InvalidInstructionData); }
        b[*o..*o + s.len()].copy_from_slice(s);
        *o += s.len();
        Ok(())
    };

    let name_bytes = name.as_bytes();
    if name_bytes.len() > 32 { return Err(ProgramError::InvalidInstructionData); }
    write(buf, offset, &(name_bytes.len() as u32).to_le_bytes())?;
    write(buf, offset, name_bytes)?;

    let symbol_bytes = symbol.as_bytes();
    if symbol_bytes.len() > 10 { return Err(ProgramError::InvalidInstructionData); }
    write(buf, offset, &(symbol_bytes.len() as u32).to_le_bytes())?;
    write(buf, offset, symbol_bytes)?;

    let uri_bytes = uri.as_bytes();
    if uri_bytes.len() > 200 { return Err(ProgramError::InvalidInstructionData); }
    write(buf, offset, &(uri_bytes.len() as u32).to_le_bytes())?;
    write(buf, offset, uri_bytes)?;

    write(buf, offset, &0u16.to_le_bytes())?;
    write(buf, offset, &[0])?; // creators None
    write(buf, offset, &[0])?; // collection None
    write(buf, offset, &[0])?; // uses None

    Ok(())
}

/// Build CreateMetadataAccountV3 instruction with fixed-size buffer
fn build_create_metadata_instruction_fixed(buf: &mut [u8], name: &str, symbol: &str, uri: &str) -> Result<usize, ProgramError> {
    let mut offset = 0usize;
    if offset + 1 > buf.len() { return Err(ProgramError::InvalidInstructionData); }
    buf[offset] = CREATE_METADATA_ACCOUNT_V3;
    offset += 1;

    serialize_metadata_v2_fixed(buf, &mut offset, name, symbol, uri)?;
    if offset + 2 > buf.len() { return Err(ProgramError::InvalidInstructionData); }
    buf[offset] = 1; offset += 1; // isMutable
    buf[offset] = 0; offset += 1; // collectionDetails None
    Ok(offset)
}

impl<'info> Initialize<'info> {
    pub fn handler(&mut self) -> Result<(), ProgramError> {
        pinocchio::msg!("Starting initialize handler");
        
        // Validate accounts
        if !self.accounts.authority.is_signer() {
            pinocchio::msg!("Authority must be signer");
            return Err(ProgramError::MissingRequiredSignature);
        }

        if !self.accounts.payer.is_signer() {
            pinocchio::msg!("Payer must be signer");
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Validate curve parameters (0=linear,1=exp,2=log,3=cpmm)
        if self.instruction_data.curve_type > 3 {
            pinocchio::msg!("Invalid curve type");
            return Err(XTokenError::InvalidCurveParameters.into());
        }

        if self.instruction_data.base_price == 0 {
            pinocchio::msg!("Base price cannot be zero");
            return Err(XTokenError::InvalidCurveParameters.into());
        }

        if self.instruction_data.max_supply == 0 {
            pinocchio::msg!("Max supply cannot be zero");
            return Err(XTokenError::InvalidCurveParameters.into());
        }

        if self.instruction_data.fee_basis_points > 1000 {
            pinocchio::msg!("Fee basis points too high");
            return Err(XTokenError::InvalidCurveParameters.into());
        }

        pinocchio::msg!("Basic validation passed, extracting metadata strings");
        
        // Extract metadata strings - this is where the panic might occur
        let token_name = self.instruction_data.get_token_name().map_err(|e| {
            pinocchio::msg!("Failed to extract token name");
            e
        })?;
        pinocchio::msg!("Token name extracted successfully");
        
        let token_symbol = self.instruction_data.get_token_symbol().map_err(|e| {
            pinocchio::msg!("Failed to extract token symbol");
            e
        })?;
        pinocchio::msg!("Token symbol extracted successfully");
        
        let token_uri = self.instruction_data.get_token_uri().map_err(|e| {
            pinocchio::msg!("Failed to extract token URI");
            e
        })?;
        pinocchio::msg!("Token URI extracted successfully");
        
        let owner_str = self.instruction_data.get_owner().map_err(|e| {
            pinocchio::msg!("Failed to extract owner");
            e
        })?;
        pinocchio::msg!("All strings extracted successfully");

        // Derive bonding curve PDA
        pinocchio::msg!("Deriving bonding curve PDA");
        let seeds = &[XToken::SEED_PREFIX, self.accounts.mint.key().as_ref()];
        let (bonding_curve_address, bump) =
            pinocchio::pubkey::find_program_address(seeds, &crate::ID);

        if bonding_curve_address != *self.accounts.bonding_curve.key() {
            pinocchio::msg!("Invalid bonding curve PDA");
            return Err(ProgramError::InvalidSeeds);
        }

        // Derive treasury PDA
        pinocchio::msg!("Deriving treasury PDA");
        let treasury_seeds = &[b"treasury", self.accounts.mint.key().as_ref()];
        let (treasury_address, treasury_bump) =
            pinocchio::pubkey::find_program_address(treasury_seeds, &crate::ID);

        if treasury_address != *self.accounts.treasury.key() {
            pinocchio::msg!("Invalid treasury PDA");
            return Err(ProgramError::InvalidSeeds);
        }

        // Derive metadata PDA and verify
        pinocchio::msg!("Deriving metadata PDA");
        let metaplex_program_id = Pubkey::from(METAPLEX_TOKEN_METADATA_ID);
        let metadata_seeds = &[
            METADATA_PREFIX,
            metaplex_program_id.as_ref(),
            self.accounts.mint.key().as_ref(),
        ];
        let (metadata_address, _metadata_bump) =
            pinocchio::pubkey::find_program_address(metadata_seeds, &metaplex_program_id);

        if metadata_address != *self.accounts.metadata_account.key() {
            pinocchio::msg!("Invalid metadata PDA");
            return Err(ProgramError::InvalidSeeds);
        }

        pinocchio::msg!("All PDAs validated, creating bonding curve account");

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

        pinocchio::msg!("Bonding curve account created, creating treasury account");

        // Create treasury PDA account (system-owned, space=0)
        let treasury_space = 0;
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

        pinocchio::msg!("Treasury account created, verifying mint");

        // Verify mint account exists (should be created by client)
        if self.accounts.mint.data_is_empty() {
            pinocchio::msg!("Mint account not initialized");
            return Err(ProgramError::UninitializedAccount);
        }

        pinocchio::msg!("Initializing mint");

        // Initialize mint with bonding curve as authority
        pinocchio_token::instructions::InitializeMint2 {
            mint: self.accounts.mint,
            decimals: self.instruction_data.decimals,
            mint_authority: &bonding_curve_address,
            freeze_authority: Some(&bonding_curve_address),
        }
        .invoke()?;

        pinocchio::msg!("Mint initialized, building metadata instruction");
        
        // Create metadata instruction with fixed-size buffer
        let mut ix_buf = [0u8; 300];
        let ix_len = build_create_metadata_instruction_fixed(
            &mut ix_buf,
            token_name,
            token_symbol,
            token_uri,
        ).map_err(|e| {
            pinocchio::msg!("Failed to build metadata instruction");
            e
        })?;

        pinocchio::msg!("Metadata instruction built successfully");
        
        // Calculate actual size of instruction data
        let actual_data_size = 1 + // discriminator
            4 + token_name.len() + // name
            4 + token_symbol.len() + // symbol  
            4 + token_uri.len() + // uri
            2 + // seller_fee_basis_points
            1 + // creators (None)
            1 + // collection (None)  
            1 + // uses (None)
            1 + // is_mutable
            1; // collection_details (None)

        // Build the instruction struct manually with only the used data
        let metadata_instruction = pinocchio::instruction::Instruction {
            program_id: &metaplex_program_id,
            accounts: &[
                AccountMeta {
                    pubkey: self.accounts.metadata_account.key(),
                    is_signer: false,
                    is_writable: true,
                },
                AccountMeta {
                    pubkey: self.accounts.mint.key(),
                    is_signer: false,
                    is_writable: false,
                },
                AccountMeta {
                    pubkey: self.accounts.bonding_curve.key(), // mint_authority
                    is_signer: true,
                    is_writable: false,
                },
                AccountMeta {
                    pubkey: self.accounts.payer.key(),
                    is_signer: true,
                    is_writable: true,
                },
                AccountMeta {
                    pubkey: self.accounts.authority.key(), // update_authority
                    is_signer: false,
                    is_writable: false,
                },
                AccountMeta {
                    pubkey: self.accounts.system_program.key(),
                    is_signer: false,
                    is_writable: false,
                },
                AccountMeta {
                    pubkey: self.accounts.rent.key(),
                    is_signer: false,
                    is_writable: false,
                },
            ],
            data: &ix_buf[..ix_len.min(actual_data_size)],
        };

        pinocchio::msg!("Preparing to invoke metadata creation");

        // Prepare signer seeds for bonding curve - use Seed array directly
        let bump_bytes = [bump];
        let signer_seeds = [
            Seed::from(XToken::SEED_PREFIX),
            Seed::from(self.accounts.mint.key().as_ref()),
            Seed::from(&bump_bytes),
        ];
        let signer = Signer::from(&signer_seeds);

        // Collect account infos for metadata creation
        let metadata_account_infos = [
            self.accounts.metadata_account,
            self.accounts.mint,
            self.accounts.bonding_curve, // mint_authority (signer)
            self.accounts.payer,
            self.accounts.authority, // update_authority
            self.accounts.system_program,
            self.accounts.rent,
        ];

        pinocchio::msg!("Invoking Metaplex metadata creation");

        // Invoke Metaplex to create metadata
        pinocchio::program::invoke_signed(
            &metadata_instruction,
            &metadata_account_infos,
            &[signer],
        ).map_err(|e| {
            pinocchio::msg!("Metaplex metadata creation failed");
            e
        })?;

        pinocchio::msg!("Metadata created successfully, initializing bonding curve state");

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
            &owner_str,
            bump,
        )?;

        pinocchio::msg!("Bonding curve initialized");

        // Optional: perform initial pre-buy similar to pump.fun
        if self.instruction_data.initial_buy_amount > 0 {
            pinocchio::msg!("Processing initial buy");
            
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
                pinocchio::msg!("Slippage exceeded");
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
                pinocchio::msg!("Treasury cap exceeded");
                return Err(ProgramError::InvalidArgument);
            }

            // Ensure payer has enough SOL
            if self.accounts.payer.lamports() < total_with_fee {
                pinocchio::msg!("Insufficient funds");
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
                pinocchio::msg!("Creating authority ATA");
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
            pinocchio::msg!("Transferring SOL to treasury");
            pinocchio_system::instructions::Transfer {
                from: self.accounts.payer,
                to: self.accounts.treasury,
                lamports: total_cost,
            }
            .invoke()?;
            
            if fee > 0 {
                pinocchio::msg!("Transferring fee");
                pinocchio_system::instructions::Transfer {
                    from: self.accounts.payer,
                    to: self.accounts.fee_recipient_account,
                    lamports: fee,
                }
                .invoke()?;
            }

            // Mint tokens to authority
            pinocchio::msg!("Minting initial tokens");
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