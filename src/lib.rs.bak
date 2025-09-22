#![no_std]
#![allow(unexpected_cfgs)]
use pinocchio::{no_allocator, nostd_panic_handler, program_entrypoint};
use processor::process_instruction;

pub mod error;
pub mod instructions;
pub mod processor;
pub mod state;

pinocchio_pubkey::declare_id!("9Tqo4t4QYLxNe5HVxWo7zaav13j4pETEtkjyKf7a2VfG");

program_entrypoint!(process_instruction);
no_allocator!();
nostd_panic_handler!();
