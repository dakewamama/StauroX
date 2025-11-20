use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::types::{NetworkHealth, SlotObservation};
use super::detector::NetworkDetector;

/// Monitors network health by tracking slot progression
pub struct HealthMonitor {
    observations: Arc<RwLock<HashMap<String, SlotObservation>>>,
    health: Arc<RwLock<NetworkHealth>>,
    detector: NetworkDetector,
    retention_seconds: u64,
}

impl HealthMonitor {
    pub fn new(stale_threshold_secs: i64, retention_seconds: u64) -> Self {
        Self {
            observations: Arc::new(RwLock::new(HashMap::new())),
            health: Arc::new(RwLock::new(NetworkHealth::Healthy)),
            detector: NetworkDetector::new(stale_threshold_secs),
            retention_seconds,
        }
    }

    pub async fn record_observation(&self, obs: SlotObservation) {
        let mut observations = self.observations.write().await;
        
        debug!("Recording slot {} from {}", obs.slot, obs.source);
        observations.insert(obs.source.clone(), obs);

        self.cleanup_old_observations(&mut observations);
    }

    pub async fn check_health(&self) -> NetworkHealth {
        let observations = self.observations.read().await;
        
        if observations.is_empty() {
            warn!("No slot observations available");
            return NetworkHealth::Halted;
        }

        let health = self.detector.detect_health(&observations);
        
        let mut current_health = self.health.write().await;
        if *current_health != health {
            match health {
                NetworkHealth::Healthy => info!("Network health: HEALTHY"),
                NetworkHealth::Forked => warn!("Network health: FORKED"),
                NetworkHealth::Halted => warn!("Network health: HALTED"),
            }
        }
        *current_health = health;
        
        health
    }

    pub async fn get_health(&self) -> NetworkHealth {
        *self.health.read().await
    }

    pub async fn get_observations(&self) -> HashMap<String, SlotObservation> {
        self.observations.read().await.clone()
    }

    fn cleanup_old_observations(&self, observations: &mut HashMap<String, SlotObservation>) {
        let cutoff = chrono::Utc::now() - chrono::Duration::seconds(self.retention_seconds as i64);
        observations.retain(|_, obs| obs.timestamp > cutoff);
    }
}

impl Default for HealthMonitor {
    fn default() -> Self {
        Self::new(5, 30)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_record_and_check_health() {
        let monitor = HealthMonitor::default();
        
        for i in 0..4 {
            monitor.record_observation(
                SlotObservation::new(12345, format!("rpc{}", i))
            ).await;
        }

        let health = monitor.check_health().await;
        assert_eq!(health, NetworkHealth::Healthy);
    }

    #[tokio::test]
    async fn test_observation_cleanup() {
        let monitor = HealthMonitor::new(5, 1);
        
        monitor.record_observation(
            SlotObservation::new(12345, "rpc1")
        ).await;

        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        monitor.record_observation(
            SlotObservation::new(12346, "rpc2")
        ).await;

        let obs = monitor.get_observations().await;
        assert_eq!(obs.len(), 1);
    }
}