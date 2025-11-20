use anchor_lang::prelude::*;

declare_id!("4DrbfPpr9j11Dc442LyjYakJR1QpE4Cq7yacA6MmMgf2");

pub mod state;
use state::*;

#[program]
pub mod staurox_program {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        bridge_program: Pubkey,
    ) -> Result<()> {
        let log = &mut ctx.accounts.verification_log;
        log.bridge_program = bridge_program;
        log.total_verifications = 0;
        log.successful = 0;
        log.failed = 0;
        log.ring_buffer_head = 0;
        log.recent = [CompactVerification::default(); 1000];
        log.bump = ctx.bumps.verification_log;
        
        msg!("Initialized verification log for bridge: {}", bridge_program);
        Ok(())
    }

    pub fn attest_verification(
        ctx: Context<AttestVerification>,
        signature: [u8; 64],
        slot: u64,
        verified: bool,
        risk_score: u8,
    ) -> Result<()> {
        let log = &mut ctx.accounts.verification_log;
        let clock = Clock::get()?;
        
        let verification = CompactVerification {
            signature,
            slot,
            verified,
            risk_score,
            timestamp: clock.unix_timestamp,
        };
        
        let index = log.ring_buffer_head as usize;
        log.recent[index] = verification;
        log.ring_buffer_head = ((log.ring_buffer_head + 1) % 100) as u16;
        
        log.total_verifications += 1;
        if verified {
            log.successful += 1;
        } else {
            log.failed += 1;
        }
        
        emit!(VerificationEvent {
            bridge_program: log.bridge_program,
            signature,
            slot,
            verified,
            risk_score,
            timestamp: clock.unix_timestamp,
        });
        
        msg!("Attested verification - Total: {}, Success: {}", 
             log.total_verifications, log.successful);
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(bridge_program: Pubkey)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = VerificationLog::LEN,
        seeds = [b"verification-log", bridge_program.as_ref()],
        bump
    )]
    pub verification_log: Account<'info, VerificationLog>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AttestVerification<'info> {
    #[account(
        mut,
        seeds = [b"verification-log", verification_log.bridge_program.as_ref()],
        bump = verification_log.bump
    )]
    pub verification_log: Account<'info, VerificationLog>,
    
    pub authority: Signer<'info>,
}

#[event]
pub struct VerificationEvent {
    pub bridge_program: Pubkey,
    pub signature: [u8; 64],
    pub slot: u64,
    pub verified: bool,
    pub risk_score: u8,
    pub timestamp: i64,
}