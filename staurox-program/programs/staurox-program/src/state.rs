use anchor_lang::prelude::*;

/// Main account storing verification history for a specific bridge
#[account]
pub struct VerificationLog {
    /// The bridge program we're tracking verifications for
    pub bridge_program: Pubkey,
    
    /// Total number of verifications performed
    pub total_verifications: u64,
    
    /// Count of successful verifications
    pub successful: u64,
    
    /// Count of failed verifications
    pub failed: u64,
    
    /// Current position in the ring buffer (0-999)
    pub ring_buffer_head: u16,
    
    /// Ring buffer of last 1000 verifications
    pub recent: [CompactVerification; 1000],
    
    /// PDA bump seed
    pub bump: u8,
}

/// Compact verification record (82 bytes)
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub struct CompactVerification {
    /// Transaction signature (64 bytes)
    pub signature: [u8; 64],
    
    /// Slot number when verified
    pub slot: u64,
    
    /// Verification result
    pub verified: bool,
    
    /// Risk score (0-255, divide by 255 for 0.0-1.0)
    pub risk_score: u8,
    
    /// Unix timestamp
    pub timestamp: i64,
}

// Manual Default implementation because [u8; 64] doesn't derive Default
impl Default for CompactVerification {
    fn default() -> Self {
        Self {
            signature: [0; 64],
            slot: 0,
            verified: false,
            risk_score: 0,
            timestamp: 0,
        }
    }
}

impl VerificationLog {
    /// Calculate total account size
    /// 8 (discriminator) + 32 + 8 + 8 + 8 + 2 + (82 * 1000) + 1 = 82,067 bytes
    pub const LEN: usize = 8 +              // Anchor discriminator
        32 +                                 // bridge_program (Pubkey)
        8 +                                  // total_verifications (u64)
        8 +                                  // successful (u64)
        8 +                                  // failed (u64)
        2 +                                  // ring_buffer_head (u16)
        (82 * 1000) +                       // recent array (82 bytes * 1000 items)
        1;                                   // bump (u8)
}

impl CompactVerification {
    /// Check if this verification slot is empty (default)
    pub fn is_empty(&self) -> bool {
        self.signature == [0; 64]
    }
    
    /// Get risk score as float (0.0 - 1.0)
    pub fn risk_score_f32(&self) -> f32 {
        self.risk_score as f32 / 255.0
    }
}