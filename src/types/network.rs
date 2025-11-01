use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Network health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkHealth {
    Healthy,
    Forked,
    Halted,
}

impl NetworkHealth {
    pub fn is_operational(&self) -> bool {
        matches!(self, NetworkHealth::Healthy)
    }

    pub fn is_degraded(&self) -> bool {
        !self.is_operational()
    }
}

/// Slot observation from a single RPC source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotObservation {
    pub slot: u64,
    pub source: String,
    pub timestamp: DateTime<Utc>,
    pub stake_percent: Option<f64>,
}

impl SlotObservation {
    pub fn new(slot: u64, source: impl Into<String>) -> Self {
        Self {
            slot,
            source: source.into(),
            timestamp: Utc::now(),
            stake_percent: None,
        }
    }

    pub fn with_stake(mut self, stake_percent: f64) -> Self {
        self.stake_percent = Some(stake_percent);
        self
    }

    pub fn age_seconds(&self) -> i64 {
        (Utc::now() - self.timestamp).num_seconds()
    }

    pub fn is_stale(&self, threshold_secs: i64) -> bool {
        self.age_seconds() > threshold_secs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_health_states() {
        assert!(NetworkHealth::Healthy.is_operational());
        assert!(!NetworkHealth::Forked.is_operational());
        assert!(NetworkHealth::Halted.is_degraded());
    }

    #[test]
    fn test_slot_observation_builder() {
        let obs = SlotObservation::new(12345, "rpc1").with_stake(25.5);
        assert_eq!(obs.slot, 12345);
        assert_eq!(obs.stake_percent, Some(25.5));
    }
}