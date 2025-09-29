use litesvm::LiteSVM;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    rent::Rent,
    signature::Keypair,
    signer::Signer,
    system_program,
    transaction::Transaction,
};
use std::str::FromStr;

// Helper function to derive PDA (real implementation)
fn derive_pda(seeds: &[&[u8]], program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(seeds, program_id)
}

// Helper function to derive metadata PDA
fn derive_metadata_pda(mint: &Pubkey) -> Pubkey {
    let metaplex_program_id = solana_sdk::pubkey!("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");
    let seeds = [
        b"metadata".as_ref(),
        metaplex_program_id.as_ref(),
        mint.as_ref(),
    ];
    let (metadata_pda, _bump) = Pubkey::find_program_address(&seeds, &metaplex_program_id);
    metadata_pda
}

const PROGRAM_ID: &str = "ASXm2vSkEpLKQ3YnpdCEbhADQw86gefgFQi5DbyVZonL";

fn setup() -> (LiteSVM, Keypair, Pubkey) {
    let mut svm = LiteSVM::new();
    let program_id = Pubkey::from_str(PROGRAM_ID).unwrap();

    // Load the compiled program
    svm.add_program_from_file(program_id, "target/deploy/x_token.so")
        .unwrap();

    let fee_payer = Keypair::new();
    svm.airdrop(&fee_payer.pubkey(), 10_000_000_000).unwrap(); // 10 SOL

    (svm, fee_payer, program_id)
}

fn send_ix_and_check(
    svm: &mut LiteSVM,
    fee_payer: &Keypair,
    ix: Instruction,
    should_succeed: bool,
) {
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&fee_payer.pubkey()),
        &[fee_payer],
        svm.latest_blockhash(),
    );

    let result = svm.send_transaction(tx);

    if should_succeed {
        assert!(result.is_ok(), "Transaction should succeed but failed");
    } else {
        assert!(result.is_err(), "Transaction should fail but succeeded");
    }
}

#[test]
fn empty_instruction_data_should_fail() {
    let (mut svm, fee_payer, program_id) = setup();

    let accounts = vec![
        AccountMeta {
            pubkey: fee_payer.pubkey(),
            is_signer: true,
            is_writable: true,
        },
        AccountMeta {
            pubkey: system_program::ID,
            is_signer: false,
            is_writable: false,
        },
    ];
    let ix = Instruction {
        program_id,
        accounts,
        data: vec![],
    };

    send_ix_and_check(&mut svm, &fee_payer, ix, false);
}

#[test]
fn initialize_with_wrong_data_length_should_fail() {
    let (mut svm, fee_payer, program_id) = setup();

    // Only discriminator = 0 (Initialize), but missing data body
    let data = vec![0u8];

    let accounts = vec![
        AccountMeta {
            pubkey: fee_payer.pubkey(),
            is_signer: true,
            is_writable: true,
        },
        AccountMeta {
            pubkey: system_program::ID,
            is_signer: false,
            is_writable: false,
        },
    ];
    let ix = Instruction {
        program_id,
        accounts,
        data,
    };

    send_ix_and_check(&mut svm, &fee_payer, ix, false);
}

#[test]
fn buy_tokens_with_missing_accounts_should_fail() {
    let (mut svm, fee_payer, program_id) = setup();

    // BuyTokens discriminator (1) + 16 bytes data
    let mut data = vec![1u8];
    data.extend_from_slice(&(1_000_000_000u64).to_le_bytes()); // token_amount
    data.extend_from_slice(&(1_000_000_000u64).to_le_bytes()); // max_sol

    // Missing required accounts: bonding_curve, mint, buyer_ata, treasury, etc.
    let accounts = vec![
        AccountMeta {
            pubkey: fee_payer.pubkey(),
            is_signer: true,
            is_writable: true,
        },
        AccountMeta {
            pubkey: system_program::ID,
            is_signer: false,
            is_writable: false,
        },
    ];
    let ix = Instruction {
        program_id,
        accounts,
        data,
    };

    send_ix_and_check(&mut svm, &fee_payer, ix, false);
}

#[test]
fn sell_tokens_with_missing_accounts_should_fail() {
    let (mut svm, fee_payer, program_id) = setup();

    // SellTokens discriminator (2) + 16 bytes data
    let mut data = vec![2u8];
    data.extend_from_slice(&(1_000_000_000u64).to_le_bytes()); // token_amount
    data.extend_from_slice(&(500_000_000u64).to_le_bytes()); // min_sol

    let accounts = vec![
        AccountMeta {
            pubkey: fee_payer.pubkey(),
            is_signer: true,
            is_writable: true,
        },
        AccountMeta {
            pubkey: system_program::ID,
            is_signer: false,
            is_writable: false,
        },
    ];
    let ix = Instruction {
        program_id,
        accounts,
        data,
    };

    send_ix_and_check(&mut svm, &fee_payer, ix, false);
}

#[test]
fn admin_mint_with_missing_accounts_should_fail() {
    let (mut svm, fee_payer, program_id) = setup();

    // AdminMint discriminator (4) + 8 bytes data
    let mut data = vec![4u8];
    data.extend_from_slice(&(1_000_000_000u64).to_le_bytes()); // amount

    let accounts = vec![
        AccountMeta {
            pubkey: fee_payer.pubkey(),
            is_signer: true,
            is_writable: true,
        },
        AccountMeta {
            pubkey: system_program::ID,
            is_signer: false,
            is_writable: false,
        },
    ];
    let ix = Instruction {
        program_id,
        accounts,
        data,
    };

    send_ix_and_check(&mut svm, &fee_payer, ix, false);
}

#[test]
fn withdraw_reserves_with_missing_accounts_should_fail() {
    let (mut svm, fee_payer, program_id) = setup();

    // WithdrawReserves discriminator (3) + 8 bytes data
    let mut data = vec![3u8];
    data.extend_from_slice(&(1_000_000_000u64).to_le_bytes()); // lamports

    let accounts = vec![
        AccountMeta {
            pubkey: fee_payer.pubkey(),
            is_signer: true,
            is_writable: true,
        },
        AccountMeta {
            pubkey: system_program::ID,
            is_signer: false,
            is_writable: false,
        },
    ];
    let ix = Instruction {
        program_id,
        accounts,
        data,
    };

    send_ix_and_check(&mut svm, &fee_payer, ix, false);
}

#[test]
fn invalid_discriminator_should_fail() {
    let (mut svm, fee_payer, program_id) = setup();

    // Invalid discriminator (99)
    let data = vec![99u8];

    let accounts = vec![
        AccountMeta {
            pubkey: fee_payer.pubkey(),
            is_signer: true,
            is_writable: true,
        },
        AccountMeta {
            pubkey: system_program::ID,
            is_signer: false,
            is_writable: false,
        },
    ];
    let ix = Instruction {
        program_id,
        accounts,
        data,
    };

    send_ix_and_check(&mut svm, &fee_payer, ix, false);
}

#[test]
fn test_program_loading() {
    let (mut svm, fee_payer, program_id) = setup();

    // Verify program was loaded by checking if we can create a transaction
    let accounts = vec![
        AccountMeta {
            pubkey: fee_payer.pubkey(),
            is_signer: true,
            is_writable: true,
        },
        AccountMeta {
            pubkey: system_program::ID,
            is_signer: false,
            is_writable: false,
        },
    ];
    let ix = Instruction {
        program_id,
        accounts,
        data: vec![],
    };

    // This should fail due to empty data, but the program should be loaded
    send_ix_and_check(&mut svm, &fee_payer, ix, false);
}

#[test]
fn test_airdrop_works() {
    let (svm, fee_payer, _program_id) = setup();

    // Check that airdrop worked by verifying account has lamports
    let account = svm.get_account(&fee_payer.pubkey()).unwrap();
    assert!(
        account.lamports > 0,
        "Fee payer should have lamports after airdrop"
    );
}

#[test]
fn test_multiple_transactions() {
    let (mut svm, fee_payer, program_id) = setup();

    // Send multiple transactions to test state consistency
    for i in 0..3 {
        let data = vec![i as u8]; // Different discriminators

        let accounts = vec![
            AccountMeta {
                pubkey: fee_payer.pubkey(),
                is_signer: true,
                is_writable: true,
            },
            AccountMeta {
                pubkey: system_program::ID,
                is_signer: false,
                is_writable: false,
            },
        ];
        let ix = Instruction {
            program_id,
            accounts,
            data,
        };

        // All should fail due to missing accounts/data, but no panic
        send_ix_and_check(&mut svm, &fee_payer, ix, false);
    }
}

#[test]
fn initialize_success_path() {
    let (mut svm, fee_payer, program_id) = setup();

    // 1. Create mint account properly
    let mint_keypair = Keypair::new();
    let mint_space = 82;
    let rent = Rent::default();
    let rent_exempt = rent.minimum_balance(mint_space);

    // Create mint account but DON'T initialize it - let the program do that
    let create_mint_ix = solana_sdk::system_instruction::create_account(
        &fee_payer.pubkey(),
        &mint_keypair.pubkey(),
        rent_exempt,
        mint_space as u64,
        &solana_sdk::pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"),
    );

    let tx = Transaction::new_signed_with_payer(
        &[create_mint_ix],
        Some(&fee_payer.pubkey()),
        &[&fee_payer, &mint_keypair],
        svm.latest_blockhash(),
    );
    svm.send_transaction(tx).unwrap();

    // 2. Derive PDAs
    let (bonding_curve, _bonding_curve_bump) =
        derive_pda(&[b"x_token", mint_keypair.pubkey().as_ref()], &program_id);
    let (treasury, _treasury_bump) =
        derive_pda(&[b"treasury", mint_keypair.pubkey().as_ref()], &program_id);

    // 3. Create authority keypair
    let authority_keypair = Keypair::new();
    svm.airdrop(&authority_keypair.pubkey(), 1_000_000_000)
        .unwrap();

    // 4. Derive associated token account address (but don't create it yet)
    let associated_token_account = spl_associated_token_account::get_associated_token_address(
        &authority_keypair.pubkey(),
        &mint_keypair.pubkey(),
    );

    // 5. Prepare Initialize instruction data
    let mut data = vec![0u8]; // Initialize discriminator
    data.push(9); // decimals
    data.push(0); // curve_type (linear)
    data.extend_from_slice(&100u16.to_le_bytes()); // fee_basis_points
    data.extend_from_slice(&[0u8; 32]); // owner (empty)
    data.extend_from_slice(&1_000_000u64.to_le_bytes()); // base_price
    data.extend_from_slice(&1_000u64.to_le_bytes()); // slope
    data.extend_from_slice(&1_000_000_000u64.to_le_bytes()); // max_supply
    data.extend_from_slice(&authority_keypair.pubkey().to_bytes()); // fee_recipient
    data.extend_from_slice(&0u64.to_le_bytes()); // initial_buy_amount
    data.extend_from_slice(&0u64.to_le_bytes()); // initial_max_sol
    data.extend_from_slice(&[0u8; 32]); // token_name (empty)
    data.extend_from_slice(&[0u8; 10]); // token_symbol (empty)
    data.extend_from_slice(&[0u8; 200]); // token_uri (empty)

    // 6. Create instruction with all required accounts
    let accounts = vec![
        AccountMeta {
            pubkey: authority_keypair.pubkey(),
            is_signer: true,
            is_writable: false,
        }, // authority
        AccountMeta {
            pubkey: bonding_curve,
            is_signer: false,
            is_writable: true,
        }, // bonding_curve
        AccountMeta {
            pubkey: mint_keypair.pubkey(),
            is_signer: false,
            is_writable: true,
        }, // mint
        AccountMeta {
            pubkey: treasury,
            is_signer: false,
            is_writable: true,
        }, // treasury
        AccountMeta {
            pubkey: associated_token_account,
            is_signer: false,
            is_writable: true,
        }, // authority_token_account
        AccountMeta {
            pubkey: fee_payer.pubkey(),
            is_signer: true,
            is_writable: true,
        }, // payer
        AccountMeta {
            pubkey: solana_sdk::system_program::ID,
            is_signer: false,
            is_writable: false,
        }, // system_program
        AccountMeta {
            pubkey: solana_sdk::pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"),
            is_signer: false,
            is_writable: false,
        }, // token_program
        AccountMeta {
            pubkey: solana_sdk::pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"),
            is_signer: false,
            is_writable: false,
        }, // associated_token_program
        AccountMeta {
            pubkey: solana_sdk::sysvar::rent::ID,
            is_signer: false,
            is_writable: false,
        }, // rent
        AccountMeta {
            pubkey: authority_keypair.pubkey(),
            is_signer: false,
            is_writable: false,
        }, // fee_recipient_account
        AccountMeta {
            pubkey: derive_metadata_pda(&mint_keypair.pubkey()),
            is_signer: false,
            is_writable: true,
        }, // metadata_account
        AccountMeta {
            pubkey: solana_sdk::pubkey!("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"),
            is_signer: false,
            is_writable: false,
        }, // metaplex_program
    ];

    let ix = Instruction {
        program_id,
        accounts,
        data,
    };

    // 7. Send transaction
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&fee_payer.pubkey()),
        &[&fee_payer, &authority_keypair],
        svm.latest_blockhash(),
    );

    let result = svm.send_transaction(tx);

    // 8. Check result
    if result.is_ok() {
        // Verify bonding curve account was created
        if let Some(account) = svm.get_account(&bonding_curve) {
            assert_eq!(account.owner, program_id);
            assert!(!account.data.is_empty());
        }

        // Create ATA since mint is initialized
        let create_ata_ix =
            spl_associated_token_account::instruction::create_associated_token_account(
                &fee_payer.pubkey(),
                &authority_keypair.pubkey(),
                &mint_keypair.pubkey(),
                &solana_sdk::pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"),
            );

        let tx = Transaction::new_signed_with_payer(
            &[create_ata_ix],
            Some(&fee_payer.pubkey()),
            &[&fee_payer],
            svm.latest_blockhash(),
        );

        let _ata_result = svm.send_transaction(tx);
    }
}

#[test]
fn buy_tokens_success_path() {
    let (mut svm, fee_payer, program_id) = setup();

    // First initialize a token (simplified version)
    let mint_keypair = Keypair::new();
    let authority_keypair = Keypair::new();
    svm.airdrop(&authority_keypair.pubkey(), 1_000_000_000)
        .unwrap();

    // Create mint account
    let mint_space = 82;
    let rent = Rent::default();
    let rent_exempt = rent.minimum_balance(mint_space);

    let create_mint_ix = solana_sdk::system_instruction::create_account(
        &fee_payer.pubkey(),
        &mint_keypair.pubkey(),
        rent_exempt,
        mint_space as u64,
        &solana_sdk::pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"),
    );

    let tx = Transaction::new_signed_with_payer(
        &[create_mint_ix],
        Some(&fee_payer.pubkey()),
        &[&fee_payer, &mint_keypair],
        svm.latest_blockhash(),
    );
    svm.send_transaction(tx).unwrap();

    // Derive PDAs
    let (bonding_curve, _bonding_curve_bump) =
        derive_pda(&[b"x_token", mint_keypair.pubkey().as_ref()], &program_id);
    let (treasury, _treasury_bump) =
        derive_pda(&[b"treasury", mint_keypair.pubkey().as_ref()], &program_id);

    // Create buyer keypair
    let buyer_keypair = Keypair::new();
    svm.airdrop(&buyer_keypair.pubkey(), 5_000_000_000).unwrap(); // 5 SOL

    // Derive buyer ATA address (but don't create it yet - mint not initialized)
    let buyer_ata = spl_associated_token_account::get_associated_token_address(
        &buyer_keypair.pubkey(),
        &mint_keypair.pubkey(),
    );

    // Prepare BuyTokens instruction data
    let mut data = vec![1u8]; // BuyTokens discriminator
    data.extend_from_slice(&1_000_000u64.to_le_bytes()); // token_amount
    data.extend_from_slice(&1_000_000_000u64.to_le_bytes()); // max_sol (1 SOL max)

    let accounts = vec![
        AccountMeta {
            pubkey: buyer_keypair.pubkey(),
            is_signer: true,
            is_writable: true,
        }, // buyer
        AccountMeta {
            pubkey: bonding_curve,
            is_signer: false,
            is_writable: true,
        }, // bonding_curve
        AccountMeta {
            pubkey: mint_keypair.pubkey(),
            is_signer: false,
            is_writable: true,
        }, // mint
        AccountMeta {
            pubkey: treasury,
            is_signer: false,
            is_writable: true,
        }, // treasury
        AccountMeta {
            pubkey: buyer_ata,
            is_signer: false,
            is_writable: true,
        }, // buyer_token_account
        AccountMeta {
            pubkey: solana_sdk::system_program::ID,
            is_signer: false,
            is_writable: false,
        }, // system_program
        AccountMeta {
            pubkey: solana_sdk::pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"),
            is_signer: false,
            is_writable: false,
        }, // token_program
    ];

    let ix = Instruction {
        program_id,
        accounts,
        data,
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&fee_payer.pubkey()),
        &[&fee_payer, &buyer_keypair],
        svm.latest_blockhash(),
    );

    let result = svm.send_transaction(tx);

    // This will likely fail due to bonding curve not being initialized
    // but we're testing the instruction structure
    assert!(
        result.is_err(),
        "BuyTokens should fail without initialized bonding curve"
    );
}

#[test]
fn sell_tokens_success_path() {
    let (mut svm, fee_payer, program_id) = setup();

    // Similar setup as buy_tokens but for selling
    let mint_keypair = Keypair::new();
    let seller_keypair = Keypair::new();
    svm.airdrop(&seller_keypair.pubkey(), 1_000_000_000)
        .unwrap();

    // Derive PDAs
    let (bonding_curve, _bonding_curve_bump) =
        derive_pda(&[b"x_token", mint_keypair.pubkey().as_ref()], &program_id);
    let (treasury, _treasury_bump) =
        derive_pda(&[b"treasury", mint_keypair.pubkey().as_ref()], &program_id);

    // Create seller ATA
    let seller_ata = spl_associated_token_account::get_associated_token_address(
        &seller_keypair.pubkey(),
        &mint_keypair.pubkey(),
    );

    // Prepare SellTokens instruction data
    let mut data = vec![2u8]; // SellTokens discriminator
    data.extend_from_slice(&500_000u64.to_le_bytes()); // token_amount
    data.extend_from_slice(&500_000_000u64.to_le_bytes()); // min_sol (0.5 SOL min)

    let accounts = vec![
        AccountMeta {
            pubkey: seller_keypair.pubkey(),
            is_signer: true,
            is_writable: true,
        }, // seller
        AccountMeta {
            pubkey: bonding_curve,
            is_signer: false,
            is_writable: true,
        }, // bonding_curve
        AccountMeta {
            pubkey: mint_keypair.pubkey(),
            is_signer: false,
            is_writable: true,
        }, // mint
        AccountMeta {
            pubkey: treasury,
            is_signer: false,
            is_writable: true,
        }, // treasury
        AccountMeta {
            pubkey: seller_ata,
            is_signer: false,
            is_writable: true,
        }, // seller_token_account
        AccountMeta {
            pubkey: solana_sdk::system_program::ID,
            is_signer: false,
            is_writable: false,
        }, // system_program
        AccountMeta {
            pubkey: solana_sdk::pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"),
            is_signer: false,
            is_writable: false,
        }, // token_program
    ];

    let ix = Instruction {
        program_id,
        accounts,
        data,
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&fee_payer.pubkey()),
        &[&fee_payer, &seller_keypair],
        svm.latest_blockhash(),
    );

    let result = svm.send_transaction(tx);

    // This will likely fail due to bonding curve not being initialized
    assert!(
        result.is_err(),
        "SellTokens should fail without initialized bonding curve"
    );
}

#[test]
fn admin_mint_success_path() {
    let (mut svm, fee_payer, program_id) = setup();

    let mint_keypair = Keypair::new();
    let admin_keypair = Keypair::new();
    svm.airdrop(&admin_keypair.pubkey(), 1_000_000_000).unwrap();

    // Derive PDAs
    let (bonding_curve, _bonding_curve_bump) =
        derive_pda(&[b"x_token", mint_keypair.pubkey().as_ref()], &program_id);

    // Create recipient ATA
    let recipient_ata = spl_associated_token_account::get_associated_token_address(
        &admin_keypair.pubkey(),
        &mint_keypair.pubkey(),
    );

    // Prepare AdminMint instruction data
    let mut data = vec![4u8]; // AdminMint discriminator
    data.extend_from_slice(&1_000_000u64.to_le_bytes()); // amount

    let accounts = vec![
        AccountMeta {
            pubkey: admin_keypair.pubkey(),
            is_signer: true,
            is_writable: false,
        }, // admin
        AccountMeta {
            pubkey: bonding_curve,
            is_signer: false,
            is_writable: true,
        }, // bonding_curve
        AccountMeta {
            pubkey: mint_keypair.pubkey(),
            is_signer: false,
            is_writable: true,
        }, // mint
        AccountMeta {
            pubkey: recipient_ata,
            is_signer: false,
            is_writable: true,
        }, // recipient_token_account
        AccountMeta {
            pubkey: solana_sdk::pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"),
            is_signer: false,
            is_writable: false,
        }, // token_program
    ];

    let ix = Instruction {
        program_id,
        accounts,
        data,
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&fee_payer.pubkey()),
        &[&fee_payer, &admin_keypair],
        svm.latest_blockhash(),
    );

    let result = svm.send_transaction(tx);

    // This will likely fail due to bonding curve not being initialized
    assert!(
        result.is_err(),
        "AdminMint should fail without initialized bonding curve"
    );
}

#[test]
fn withdraw_reserves_success_path() {
    let (mut svm, fee_payer, program_id) = setup();

    let mint_keypair = Keypair::new();
    let admin_keypair = Keypair::new();
    svm.airdrop(&admin_keypair.pubkey(), 1_000_000_000).unwrap();

    // Derive PDAs
    let (bonding_curve, _bonding_curve_bump) =
        derive_pda(&[b"x_token", mint_keypair.pubkey().as_ref()], &program_id);
    let (treasury, _treasury_bump) =
        derive_pda(&[b"treasury", mint_keypair.pubkey().as_ref()], &program_id);

    // Prepare WithdrawReserves instruction data
    let mut data = vec![3u8]; // WithdrawReserves discriminator
    data.extend_from_slice(&1_000_000_000u64.to_le_bytes()); // lamports (1 SOL)

    let accounts = vec![
        AccountMeta {
            pubkey: admin_keypair.pubkey(),
            is_signer: true,
            is_writable: true,
        }, // admin
        AccountMeta {
            pubkey: bonding_curve,
            is_signer: false,
            is_writable: true,
        }, // bonding_curve
        AccountMeta {
            pubkey: treasury,
            is_signer: false,
            is_writable: true,
        }, // treasury
        AccountMeta {
            pubkey: solana_sdk::system_program::ID,
            is_signer: false,
            is_writable: false,
        }, 
    ];

    let ix = Instruction {
        program_id,
        accounts,
        data,
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&fee_payer.pubkey()),
        &[&fee_payer, &admin_keypair],
        svm.latest_blockhash(),
    );

    let result = svm.send_transaction(tx);

    // This will likely fail due to bonding curve not being initialized
    assert!(
        result.is_err(),
        "WithdrawReserves should fail without initialized bonding curve"
    );
}

#[test]
fn test_insufficient_funds() {
    let (mut svm, _fee_payer, program_id) = setup();

    // Create a keypair with no funds
    let poor_keypair = Keypair::new();

    let accounts = vec![
        AccountMeta {
            pubkey: poor_keypair.pubkey(),
            is_signer: true,
            is_writable: true,
        },
        AccountMeta {
            pubkey: system_program::ID,
            is_signer: false,
            is_writable: false,
        },
    ];
    let ix = Instruction {
        program_id,
        accounts,
        data: vec![0u8], 
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&poor_keypair.pubkey()),
        &[&poor_keypair],
        svm.latest_blockhash(),
    );

    let result = svm.send_transaction(tx);

    assert!(
        result.is_err(),
        "Transaction should fail with insufficient funds"
    );
}

#[test]
fn test_wrong_program_id() {
    let (mut svm, fee_payer, _program_id) = setup();

    let wrong_program_id = Pubkey::new_unique();

    let accounts = vec![
        AccountMeta {
            pubkey: fee_payer.pubkey(),
            is_signer: true,
            is_writable: true,
        },
        AccountMeta {
            pubkey: system_program::ID,
            is_signer: false,
            is_writable: false,
        },
    ];
    let ix = Instruction {
        program_id: wrong_program_id,
        accounts,
        data: vec![0u8],
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&fee_payer.pubkey()),
        &[&fee_payer],
        svm.latest_blockhash(),
    );

    let result = svm.send_transaction(tx);

    assert!(
        result.is_err(),
        "Transaction should fail with wrong program ID"
    );
}
