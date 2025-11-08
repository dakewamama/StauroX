use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use tracing::debug;

use crate::error::Result;
use crate::types::FinalityLevel;

/// Finality checker - determines if transaction is safely finalized
pub struct FinalityChecker {
    commitment: CommitmentConfig,
}

impl FinalityChecker {
    pub fn new() -> Self {
        Self {
            commitment: CommitmentConfig::finalized(),
        }
    }

    /// Check finality level based on confirmations
    pub async fn check_finality(
        &self,
        client: &RpcClient,
        slot: u64,
    ) -> Result<FinalityLevel> {
        // Get current slot
        let current_slot = client.get_slot()?;
        let confirmations = current_slot.saturating_sub(slot);

        debug!("Slot {} has {} confirmations", slot, confirmations);

        // Determine finality level based on confirmations
        let finality = match confirmations {
            0..=31 => FinalityLevel::Fast,      // <32 confirmations
            32..=63 => FinalityLevel::Safe,     // 32-63 confirmations  
            _ => FinalityLevel::UltraSafe,      // 64+ confirmations
        };

        Ok(finality)
    }

    /// Check if transaction is in a finalized block
    pub fn is_finalized(&self, client: &RpcClient, slot: u64) -> Result<bool> {
        // Query with finalized commitment
        match client.get_block_with_config(
            slot,
            solana_client::rpc_config::RpcBlockConfig {
                commitment: Some(self.commitment),
                ..Default::default()
            },
        ) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

impl Default for FinalityChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_finality_checker_creation() {
        let checker = FinalityChecker::new();
        assert_eq!(checker.commitment, CommitmentConfig::finalized());
    }
}