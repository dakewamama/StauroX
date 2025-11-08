use std::sync::Arc;
use tokio::time;
use tracing::{error, info};

use crate::api::{create_router, ws_handler, ApiState, WsState};
use crate::config::Config;
use crate::error::Result;
use crate::monitor::HealthMonitor;
use crate::rpc::MultiRpcClient;
use crate::types::SlotObservation;
use crate::verification::VerificationEngine;

/// Main verification service
pub struct VerificationService {
    config: Arc<Config>,
    health_monitor: Arc<HealthMonitor>,
    rpc_client: Arc<MultiRpcClient>,
    verification_engine: Arc<VerificationEngine>,
    ws_state: WsState,
    health_check_counter: Arc<std::sync::atomic::AtomicU64>,
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

        let verification_engine = Arc::new(VerificationEngine::new(
            Arc::clone(&rpc_client),
            Arc::clone(&health_monitor),
        ));

        let ws_state = WsState::new();

        Ok(Self {
            config: Arc::new(config),
            health_monitor,
            rpc_client,
            verification_engine,
            ws_state,
            health_check_counter: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        })
    }

    /// Start the verification service with API servers
    pub async fn run(self: Arc<Self>) -> Result<()> {
        info!("Starting StauroX Verification Service");
        info!("Network: {}", self.config.network.name());
        info!(
            "Configured with {} RPC endpoints",
            self.rpc_client.client_count()
        );
        info!(
            "Consensus threshold: {}/{}",
            self.rpc_client.consensus_threshold(),
            self.rpc_client.client_count()
        );

        // Start health monitoring
        let health_task = {
            let service = Arc::clone(&self);
            tokio::spawn(async move {
                service.start_health_monitoring().await
            })
        };

        // Start REST API
        let rest_task = {
            let service = Arc::clone(&self);
            tokio::spawn(async move {
                service.start_rest_api().await
            })
        };

        // Start WebSocket server
        let ws_task = {
            let service = Arc::clone(&self);
            tokio::spawn(async move {
                service.start_websocket_server().await
            })
        };

        // Wait for all tasks
        let (health_result, rest_result, ws_result) = tokio::try_join!(health_task, rest_task, ws_task)
            .map_err(|e| crate::error::StauroXError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Task error: {}", e)
            )))?;

        // Handle individual task results
        health_result?;
        rest_result?;
        ws_result?;

        Ok(())
    }

    /// Start REST API server
    async fn start_rest_api(&self) -> Result<()> {
        let api_state = ApiState {
            engine: Arc::clone(&self.verification_engine),
        };

        let app = create_router(api_state);

        let addr = format!("0.0.0.0:{}", self.config.api.rest_port);
        info!("REST API listening on {}", addr);

        let listener = tokio::net::TcpListener::bind(&addr)
            .await
            .map_err(|e| crate::error::StauroXError::Io(e))?;

        axum::serve(listener, app)
            .await
            .map_err(|e| crate::error::StauroXError::Io(
                std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
            ))?;

        Ok(())
    }

    /// Start WebSocket server
    async fn start_websocket_server(&self) -> Result<()> {
        use axum::routing::get;

        let ws_state = self.ws_state.clone();
        let app = axum::Router::new()
            .route("/events", get(ws_handler))
            .with_state(ws_state);

        let addr = format!("0.0.0.0:{}", self.config.api.websocket_port);
        info!("WebSocket server listening on {}", addr);

        let listener = tokio::net::TcpListener::bind(&addr)
            .await
            .map_err(|e| crate::error::StauroXError::Io(e))?;

        axum::serve(listener, app)
            .await
            .map_err(|e| crate::error::StauroXError::Io(
                std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
            ))?;

        Ok(())
    }

    /// Main health monitoring loop
    async fn start_health_monitoring(&self) -> Result<()> {
        let mut interval = time::interval(self.config.health_check_interval());

        info!("Starting network health monitoring...");

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
        
        // Increment counter
        let count = self.health_check_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        
        // Only log every 100 checks (~40 seconds) or on first check
        if count == 0 || count % 100 == 0 {
            info!("Health: {:?} | Slot: {} | Uptime: {}s", health, slot, count);
        }

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