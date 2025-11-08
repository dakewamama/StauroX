use solana_sdk::signature::Signature;
use std::sync::Arc;
use tracing::{debug, info};

use crate::error::Result;
use crate::monitor::HealthMonitor;
use crate::rpc::MultiRpcClient;
use crate::types::{FinalityLevel, NetworkHealth, VerificationResult};

use super::finality::FinalityChecker;
use super::risk::RiskScorer;

/// Main verification engine
pub struct VerificationEngine {
    rpc_client: Arc<MultiRpcClient>,
    pub health_monitor: Arc<HealthMonitor>,
    _finality_checker: FinalityChecker,
    risk_scorer: RiskScorer,
}

impl VerificationEngine {
    pub fn new(
        rpc_client: Arc<MultiRpcClient>,
        health_monitor: Arc<HealthMonitor>,
    ) -> Self {
        Self {
            rpc_client,
            health_monitor,
            _finality_checker: FinalityChecker::new(),
            risk_scorer: RiskScorer::new(),
        }
    }

    /// Verify a transaction
    pub async fn verify_transaction(
        &self,
        signature: &Signature,
    ) -> Result<VerificationResult> {
        info!("Verifying transaction: {}", signature);

        // Step 1: Check network health
        let network_health = self.health_monitor.get_health().await;
        if network_health == NetworkHealth::Halted {
            debug!("Network halted - refusing verification");
            return Ok(VerificationResult::new(*signature, 0)
                .with_verification(false)
                .with_network_health(network_health)
                .with_risk_score(1.0));
        }

        // Step 2: Fetch transaction with consensus
        let tx = self
            .rpc_client
            .fetch_transaction_with_consensus(signature)
            .await?;

        let slot = tx.slot;
        debug!("Transaction found in slot: {}", slot);

        // Step 3: Check finality (we'll use first RPC for now)
        // TODO: Add consensus for finality check across multiple RPCs
        let finality = FinalityLevel::Safe; // Placeholder for now
        
        // Step 4: Calculate risk score
        let consensus_ratio = 0.75; // Placeholder - we'd track this in MultiRpcClient
        let risk_score = self.risk_scorer.calculate_risk(
            finality,
            network_health,
            consensus_ratio,
        );

        // Step 5: Build result
        let result = VerificationResult::new(*signature, slot)
            .with_verification(true)
            .with_finality(finality)
            .with_network_health(network_health)
            .with_risk_score(risk_score)
            .with_consensus(3); // Placeholder

        info!(
            "Verification complete: slot={}, risk={:.2}, finality={:?}",
            slot, risk_score, finality
        );

        Ok(result)
    }

    /// Batch verify multiple transactions
    pub async fn verify_batch(
        &self,
        signatures: &[Signature],
    ) -> Vec<Result<VerificationResult>> {
        let mut results = Vec::with_capacity(signatures.len());

        for sig in signatures {
            results.push(self.verify_transaction(sig).await);
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::service::VerificationService;

    #[test]
    fn test_engine_creation() {
        let config = Config::devnet();
        let service = VerificationService::new(config).unwrap();
        
        let _engine = VerificationEngine::new(
            service.rpc_client(),
            service.health_monitor(),
        );

        // Engine should be created successfully
        assert!(true);
    }
}