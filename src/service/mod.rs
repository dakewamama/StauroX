use std::sync::Arc;
use tokio::time;
use tracing::{error, info};

use crate::config::Config;
use crate::error::Result;
use crate::monitor::HealthMonitor;
use crate::rpc::MultiRpcClient;
use crate::types::SlotObservation;

/// Main verification service
pub struct VerificationService {
    config: Arc<Config>,
    health_monitor: Arc<HealthMonitor>,
    rpc_client: Arc<MultiRpcClient>,
}

impl VerificationService {
    pub fn new(config: Config) -> Result<Self> {
        config.validate()?;

        let health_monitor = Arc::new(HealthMonitor::new(
            config.monitoring.stale_threshold_seconds,
            config.monitoring.slot_retention_seconds,
        ));

        let rpc_client = Arc::new(MultiRpcClient::new(
            config.rpc.endpoints.clone(),
            config.rpc.consensus_threshold,
        ));

        Ok(Self {
            config: Arc::new(config),
            health_monitor,
            rpc_client,
        })
    }

    /// Start the verification service
    pub async fn run(self: Arc<Self>) -> Result<()> {
        info!("🚀 Starting StauroX Verification Service");
        info!("🌐 Network: {}", self.config.network.name());
        info!(
            "📡 Configured with {} RPC endpoints",
            self.rpc_client.client_count()
        );
        info!(
            "🔍 Consensus threshold: {}/{}",
            self.rpc_client.consensus_threshold(),
            self.rpc_client.client_count()
        );

        self.start_health_monitoring().await
    }

    /// Main health monitoring loop
    async fn start_health_monitoring(&self) -> Result<()> {
        let mut interval = time::interval(self.config.health_check_interval());

        info!("👀 Starting network health monitoring...");

        loop {
            interval.tick().await;

            if let Err(e) = self.health_check_cycle().await {
                error!("Health check error: {}", e);
            }
        }
    }

    /// Single health check cycle
    async fn health_check_cycle(&self) -> Result<()> {
        let slot = self.rpc_client.get_slot_with_consensus().await?;

        self.health_monitor
            .record_observation(SlotObservation::new(slot, "consensus"))
            .await;

        let health = self.health_monitor.check_health().await;

        info!("Network: {:?} | Slot: {}", health, slot);

        Ok(())
    }

    pub fn health_monitor(&self) -> Arc<HealthMonitor> {
        Arc::clone(&self.health_monitor)
    }

    pub fn rpc_client(&self) -> Arc<MultiRpcClient> {
        Arc::clone(&self.rpc_client)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_creation() {
        let config = Config::default();
        let service = VerificationService::new(config);
        assert!(service.is_ok());
    }

    #[test]
    fn test_service_rejects_invalid_config() {
        let mut config = Config::default();
        config.rpc.endpoints.clear();
        
        let service = VerificationService::new(config);
        assert!(service.is_err());
    }
}