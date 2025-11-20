use crate::types::{FinalityLevel, NetworkHealth};

/// Risk scorer - calculates risk score for verification
pub struct RiskScorer;

impl RiskScorer {
    pub fn new() -> Self {
        Self
    }

    /// Calculate risk score (0.0 = no risk, 1.0 = maximum risk)
    pub fn calculate_risk(
        &self,
        finality: FinalityLevel,
        network_health: NetworkHealth,
        consensus_ratio: f64,
    ) -> f64 {
        let mut risk = 0.0;

        // Finality risk
        risk += match finality {
            FinalityLevel::UltraSafe => 0.01,
            FinalityLevel::Safe => 0.05,
            FinalityLevel::Fast => 0.15,
        };

        // Network health risk
        risk += match network_health {
            NetworkHealth::Healthy => 0.0,
            NetworkHealth::Forked => 0.3,
            NetworkHealth::Halted => 0.5,
        };

        // Consensus risk (lower consensus = higher risk)
        let consensus_risk = 1.0 - consensus_ratio;
        risk += consensus_risk * 0.2;

        // Clamp to [0.0, 1.0]
        risk.clamp(0.0, 1.0)
    }

    /// Determine if risk is acceptable
    pub fn is_acceptable_risk(&self, risk_score: f64, threshold: f64) -> bool {
        risk_score <= threshold
    }
}

impl Default for RiskScorer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ultra_safe_risk() {
        let scorer = RiskScorer::new();
        let risk = scorer.calculate_risk(
            FinalityLevel::UltraSafe,
            NetworkHealth::Healthy,
            1.0, // Perfect consensus
        );
        assert!(risk < 0.1); // Very low risk
    }

    #[test]
    fn test_forked_network_risk() {
        let scorer = RiskScorer::new();
        let risk = scorer.calculate_risk(
            FinalityLevel::Fast,
            NetworkHealth::Forked,
            0.75,
        );
        assert!(risk > 0.3); // High risk due to fork
    }

    #[test]
    fn test_risk_threshold() {
        let scorer = RiskScorer::new();
        assert!(scorer.is_acceptable_risk(0.1, 0.2));
        assert!(!scorer.is_acceptable_risk(0.3, 0.2));
    }
}