use solana_sdk::signature::Signature;
use solana_transaction_status::EncodedConfirmedTransactionWithStatusMeta;
use std::sync::Arc;
use tracing::{debug, info, warn};

use crate::error::{Result, StauroXError};
use crate::monitor::HealthMonitor;
use crate::parsers::TransactionParser;
use crate::rpc::MultiRpcClient;
use crate::types::{FinalityLevel, NetworkHealth, VerificationResult};

use super::finality::FinalityChecker;
use super::risk::RiskScorer;

/// Main verification engine - orchestrates the complete verification pipeline
pub struct VerificationEngine {
    rpc_client: Arc<MultiRpcClient>,
    pub health_monitor: Arc<HealthMonitor>,
    _finality_checker: FinalityChecker,
    risk_scorer: RiskScorer,
    parser: TransactionParser,
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
            parser: TransactionParser::new(),
        }
    }

    /// Main verification entry point
    /// 
    /// Verification Pipeline:
    /// 1. Check network health (refuse if halted)
    /// 2. Fetch transaction from multiple RPCs with consensus
    /// 3. Parse bridge transaction (if applicable)
    /// 4. Verify transaction succeeded on-chain
    /// 5. Determine finality level based on slot age
    /// 6. Calculate risk score
    /// 7. Return verification result
    pub async fn verify_transaction(
        &self,
        signature: &Signature,
    ) -> Result<VerificationResult> {
        info!("Starting verification for: {}", signature);

        // Step 1: Network Health Check
        let network_health = self.check_network_health().await?;
        
        // Step 2: Fetch Transaction with Consensus
        let (tx, consensus_count) = self.fetch_transaction_with_metadata(signature).await?;
        
        // Step 2.5: Parse bridge transaction (NEW)
        info!("Attempting to parse bridge transaction...");
        let parsed_tx = match self.parser.parse_transaction(&tx) {
            Ok(Some(parsed)) => {
                info!(
                    "✓ Parsed {} bridge: amount={:?}, target_chain={:?}",
                    parsed.bridge_name(),
                    parsed.amount(),
                    parsed.target_chain()
                );
                Some(parsed)
            }
            Ok(None) => {
                info!("Not a bridge transaction");
                None
            }
            Err(e) => {
                warn!("Failed to parse transaction: {}", e);
                None
            }
        };
        
        // Step 3: Verify Transaction Success
        let tx_success = self.check_transaction_success(&tx)?;
        
        if !tx_success {
            return self.build_failed_verification_result(
                *signature,
                tx.slot,
                network_health,
                "Transaction failed on-chain",
                None,
            );
        }

        // Step 4: Determine Finality
        let finality = self.determine_finality_level(tx.slot).await?;
        
        // Step 5: Calculate Risk Score
        let consensus_ratio = self.calculate_consensus_ratio(consensus_count);
        let risk_score = self.calculate_risk(finality, network_health, consensus_ratio);
        
        // Step 6: Build Success Result
        let result = VerificationResult::new(*signature, tx.slot)
            .with_verification(true)
            .with_finality(finality)
            .with_network_health(network_health)
            .with_risk_score(risk_score)
            .with_consensus(consensus_count as u8)
            .with_parsed_transaction(parsed_tx);

        info!(
            "✓ Verification complete: slot={}, finality={:?}, risk={:.3}",
            tx.slot, finality, risk_score
        );

        Ok(result)
    }

    /// Step 1: Check network health
    async fn check_network_health(&self) -> Result<NetworkHealth> {
        let health = self.health_monitor.get_health().await;
        
        if health == NetworkHealth::Halted {
            warn!("Network is halted - refusing verification");
            return Err(StauroXError::verification(
                "Network halted - cannot verify transactions"
            ));
        }

        debug!("Network health: {:?}", health);
        Ok(health)
    }

    /// Step 2: Fetch transaction with consensus tracking
    async fn fetch_transaction_with_metadata(
        &self,
        signature: &Signature,
    ) -> Result<(EncodedConfirmedTransactionWithStatusMeta, usize)> {
        let tx = self
            .rpc_client
            .fetch_transaction_with_consensus(signature)
            .await?;

        let consensus_count = 1;
        
        debug!(
            "Transaction fetched: slot={}, consensus={}/{}",
            tx.slot,
            consensus_count,
            self.rpc_client.client_count()
        );

        Ok((tx, consensus_count))
    }

    /// Step 3: Check if transaction succeeded on-chain
    fn check_transaction_success(
        &self,
        tx: &EncodedConfirmedTransactionWithStatusMeta,
    ) -> Result<bool> {
        let success = tx.transaction.meta.as_ref()
            .map(|meta| meta.err.is_none())
            .unwrap_or(false);

        if success {
            debug!("Transaction succeeded on-chain");
        } else {
            warn!("Transaction failed on-chain");
        }

        Ok(success)
    }

    /// Step 4: Determine finality level based on slot age
    async fn determine_finality_level(&self, tx_slot: u64) -> Result<FinalityLevel> {
        let current_slot = self.rpc_client.get_slot_with_consensus().await?;
        let slot_age = current_slot.saturating_sub(tx_slot);

        let finality = match slot_age {
            0..=31 => FinalityLevel::Fast,
            32..=63 => FinalityLevel::Safe,
            _ => FinalityLevel::UltraSafe,
        };

        debug!(
            "Finality: {:?} (slot_age={}, current={}, tx={})",
            finality, slot_age, current_slot, tx_slot
        );

        Ok(finality)
    }

    /// Step 5: Calculate consensus ratio
    fn calculate_consensus_ratio(&self, consensus_count: usize) -> f64 {
        let total_rpcs = self.rpc_client.client_count();
        consensus_count as f64 / total_rpcs as f64
    }

    /// Step 5: Calculate risk score
    fn calculate_risk(
        &self,
        finality: FinalityLevel,
        network_health: NetworkHealth,
        consensus_ratio: f64,
    ) -> f64 {
        self.risk_scorer.calculate_risk(finality, network_health, consensus_ratio)
    }

    /// Build result for failed verification
    fn build_failed_verification_result(
        &self,
        signature: Signature,
        slot: u64,
        network_health: NetworkHealth,
        reason: &str,
        parsed_tx: Option<crate::parsers::ParsedTransaction>,
    ) -> Result<VerificationResult> {
        warn!("Verification failed: {}", reason);
        
        Ok(VerificationResult::new(signature, slot)
            .with_verification(false)
            .with_finality(FinalityLevel::Fast)
            .with_network_health(network_health)
            .with_risk_score(1.0)
            .with_consensus(0u8)
            .with_parsed_transaction(parsed_tx))
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
        let config = Config::mainnet();
        let service = VerificationService::new(config).unwrap();
        
        let _engine = VerificationEngine::new(
            service.rpc_client(),
            service.health_monitor(),
        );
    }

    #[test]
    fn test_consensus_ratio_calculation() {
        let config = Config::mainnet();
        let service = VerificationService::new(config).unwrap();
        
        let engine = VerificationEngine::new(
            service.rpc_client(),
            service.health_monitor(),
        );

        assert_eq!(engine.calculate_consensus_ratio(2), 1.0);
        assert_eq!(engine.calculate_consensus_ratio(1), 0.5);
    }
}