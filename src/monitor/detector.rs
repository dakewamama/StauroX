use std::collections::HashMap;

use crate::types::{NetworkHealth, SlotObservation};

// Constants for network health detection
const FORK_SUPPORT_THRESHOLD: f64 = 30.0;
const HEALTHY_LAG_TOLERANCE: u64 = 2;

// Detector for network forks and halts
pub struct NetworkDetector {
    stale_threshold_secs: i64,
}

impl NetworkDetector {
    pub fn new(stale_threshold_secs: i64) -> Self {
        Self {
            stale_threshold_secs,
        }
    }

    // Determine network health from observations
    pub fn detect_health(
        &self,
        observations: &HashMap<String, SlotObservation>,
    ) -> NetworkHealth {
        if observations.is_empty() {
            return NetworkHealth::Halted;
        }

        if self.all_observations_stale(observations) {
            return NetworkHealth::Halted;
        }

        if self.has_significant_fork(observations) {
            return NetworkHealth::Forked;
        }

        NetworkHealth::Healthy
    }

    fn all_observations_stale(&self, observations: &HashMap<String, SlotObservation>) -> bool {
        observations
            .values()
            .all(|obs| obs.is_stale(self.stale_threshold_secs))
    }

    fn has_significant_fork(&self, observations: &HashMap<String, SlotObservation>) -> bool {
        let slot_groups = self.group_by_slot(observations);

        if self.within_healthy_tolerance(&slot_groups) {
            return false;
        }

        let total_sources = observations.len();
        let significant_forks = slot_groups
            .values()
            .filter(|sources| {
                let support_percent = (**sources as f64 / total_sources as f64) * 100.0;
                support_percent > FORK_SUPPORT_THRESHOLD
            })
            .count();

        significant_forks > 1
    }

    fn group_by_slot(&self, observations: &HashMap<String, SlotObservation>) -> HashMap<u64, usize> {
        let mut slot_counts: HashMap<u64, usize> = HashMap::new();
        for obs in observations.values() {
            *slot_counts.entry(obs.slot).or_insert(0) += 1;
        }
        slot_counts
    }

    fn within_healthy_tolerance(&self, slot_groups: &HashMap<u64, usize>) -> bool {
        if slot_groups.len() <= 1 {
            return true;
        }

        let slots: Vec<u64> = slot_groups.keys().copied().collect();
        let min_slot = *slots.iter().min().unwrap();
        let max_slot = *slots.iter().max().unwrap();

        (max_slot - min_slot) <= HEALTHY_LAG_TOLERANCE
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_observation(slot: u64, source: &str) -> (String, SlotObservation) {
        (
            source.to_string(),
            SlotObservation {
                slot,
                source: source.to_string(),
                timestamp: Utc::now(),
                stake_percent: Some(25.0),
            },
        )
    }

    #[test]
    fn test_healthy_network() {
        let detector = NetworkDetector::new(5);
        let mut obs = HashMap::new();
        
        for i in 0..4 {
            let (source, observation) = create_observation(12345, &format!("rpc{}", i));
            obs.insert(source, observation);
        }

        assert_eq!(detector.detect_health(&obs), NetworkHealth::Healthy);
    }

    #[test]
    fn test_forked_network() {
        let detector = NetworkDetector::new(5);
        let mut obs = HashMap::new();

        for i in 0..2 {
            let (source, observation) = create_observation(100, &format!("rpc{}", i));
            obs.insert(source, observation);
        }
        for i in 2..4 {
            let (source, observation) = create_observation(105, &format!("rpc{}", i));
            obs.insert(source, observation);
        }

        assert_eq!(detector.detect_health(&obs), NetworkHealth::Forked);
    }

    #[test]
    fn test_halted_network() {
        let detector = NetworkDetector::new(5);
        let mut obs = HashMap::new();

        for i in 0..4 {
            let mut observation = SlotObservation::new(12345, format!("rpc{}", i));
            observation.timestamp = Utc::now() - chrono::Duration::seconds(10);
            obs.insert(format!("rpc{}", i), observation);
        }

        assert_eq!(detector.detect_health(&obs), NetworkHealth::Halted);
    }
}