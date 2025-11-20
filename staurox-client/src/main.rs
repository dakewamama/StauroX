use anchor_client::solana_sdk::commitment_config::CommitmentConfig;
use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_client::solana_sdk::signature::{Keypair, read_keypair_file};
use anchor_client::solana_sdk::signer::Signer;
use anchor_client::solana_sdk::system_program;
use anchor_client::{Client, Cluster, Program};
use anyhow::Result;
use std::rc::Rc;
use std::str::FromStr;

const PROGRAM_ID: &str = "4DrbfPpr9j11Dc442LyjYakJR1QpE4Cq7yacA6MmMgf2";
const WORMHOLE_BRIDGE: &str = "worm2ZoG2kUd4vFXhvjh93UUH596ayRfgQ2MgjNMTth";

fn main() -> Result<()> {
    println!(" StauroX Test Client \n");

    // Load wallet
    let payer = read_keypair_file(
        shellexpand::tilde("~/.config/solana/id.json").to_string()
    )?;
    println!("Wallet: {}", payer.pubkey());

    // Connect to devnet
    let client = Client::new_with_options(
        Cluster::Devnet,
        Rc::new(payer),
        CommitmentConfig::confirmed(),
    );

    let program_id = Pubkey::from_str(PROGRAM_ID)?;
    let program = client.program(program_id)?;
    
    // Test 1: Initialize verification log for Wormhole
    test_initialize(&program)?;
    
    // Test 2: Attest a verification
    test_attest_verification(&program)?;
    
    // Test 3: Query the verification log
    test_query_log(&program)?;

    println!("\n All tests passed!");
    Ok(())
}

fn test_initialize(program: &Program<Rc<Keypair>>) -> Result<()> {
    println!("Test 1: Initialize Verification Log");
    println!("-----------------------------------");
    
    let wormhole = Pubkey::from_str(WORMHOLE_BRIDGE)?;
    let (pda, _bump) = Pubkey::find_program_address(
        &[b"verification-log", wormhole.as_ref()],
        &program.id(),
    );
    
    println!("Wormhole bridge: {}", wormhole);
    println!("PDA address: {}", pda);
    
    // Check if already initialized
    match program.account::<VerificationLog>(pda) {
        Ok(log) => {
            println!("✓ Already initialized");
            println!("  Total verifications: {}", log.total_verifications);
            println!("  Successful: {}", log.successful);
            println!("  Failed: {}", log.failed);
            return Ok(());
        }
        Err(_) => {
            println!("Not initialized yet, creating...");
        }
    }
    
    // Initialize
    let sig = program
        .request()
        .accounts(staurox_program::accounts::Initialize {
            verification_log: pda,
            authority: program.payer(),
            system_program: system_program::id(),
        })
        .args(staurox_program::instruction::Initialize {
            bridge_program: wormhole,
        })
        .send()?;
    
    println!("✓ Initialized! Signature: {}", sig);
    Ok(())
}

fn test_attest_verification(program: &Program<Rc<Keypair>>) -> Result<()> {
    println!("\nTest 2: Attest Verification");
    println!("----------------------------");
    
    let wormhole = Pubkey::from_str(WORMHOLE_BRIDGE)?;
    let (pda, _bump) = Pubkey::find_program_address(
        &[b"verification-log", wormhole.as_ref()],
        &program.id(),
    );
    
    // Create a test signature (64 bytes)
    let mut test_signature = [0u8; 64];
    test_signature[0] = 0xAB;
    test_signature[1] = 0xCD;
    test_signature[63] = 0xFF;
    
    let sig = program
        .request()
        .accounts(staurox_program::accounts::AttestVerification {
            verification_log: pda,
            authority: program.payer(),
        })
        .args(staurox_program::instruction::AttestVerification {
            signature: test_signature,
            slot: 12345,
            verified: true,
            risk_score: 10, // 10/255 = 0.039 risk
        })
        .send()?;
    
    println!("✓ Attested verification! Signature: {}", sig);
    Ok(())
}

fn test_query_log(program: &Program<Rc<Keypair>>) -> Result<()> {
    println!("\nTest 3: Query Verification Log");
    println!("-------------------------------");
    
    let wormhole = Pubkey::from_str(WORMHOLE_BRIDGE)?;
    let (pda, _bump) = Pubkey::find_program_address(
        &[b"verification-log", wormhole.as_ref()],
        &program.id(),
    );
    
    let log: VerificationLog = program.account(pda)?;
    
    println!("Bridge: {}", log.bridge_program);
    println!("Total verifications: {}", log.total_verifications);
    println!("Successful: {}", log.successful);
    println!("Failed: {}", log.failed);
    println!("Ring buffer head: {}", log.ring_buffer_head);
    
    println!("\nRecent verifications:");
    for (i, v) in log.recent.iter().take(5).enumerate() {
        if v.slot > 0 {
            println!("  [{}] Slot: {}, Verified: {}, Risk: {:.3}", 
                i, v.slot, v.verified, v.risk_score as f32 / 255.0);
        }
    }
    
    Ok(())
}

// Define the account structures (must match the program)
use anchor_lang::prelude::*;

#[account]
pub struct VerificationLog {
    pub bridge_program: Pubkey,
    pub total_verifications: u64,
    pub successful: u64,
    pub failed: u64,
    pub ring_buffer_head: u16,
    pub recent: [CompactVerification; 100],
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub struct CompactVerification {
    pub signature: [u8; 64],
    pub slot: u64,
    pub verified: bool,
    pub risk_score: u8,
    pub timestamp: i64,
}

// Mock the program module
mod staurox_program {
    use super::*;
    
    pub mod accounts {
        use super::*;
        
        #[derive(anchor_lang::Accounts)]
        pub struct Initialize {
            pub verification_log: Pubkey,
            pub authority: Pubkey,
            pub system_program: Pubkey,
        }
        
        #[derive(anchor_lang::Accounts)]
        pub struct AttestVerification {
            pub verification_log: Pubkey,
            pub authority: Pubkey,
        }
    }
    
    pub mod instruction {
        use super::*;
        
        #[derive(AnchorSerialize)]
        pub struct Initialize {
            pub bridge_program: Pubkey,
        }
        
        #[derive(AnchorSerialize)]
        pub struct AttestVerification {
            pub signature: [u8; 64],
            pub slot: u64,
            pub verified: bool,
            pub risk_score: u8,
        }
    }
}