use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use solana_sdk::signature::Signature;

use super::network::NetworkHealth;

/// Finality confidence levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum FinalityLevel {
    Fast,      // 66% stake
    Safe,      // 80% stake
    UltraSafe, // 90% stake
}

impl FinalityLevel {
    pub fn required_stake_percent(&self) -> f64 {
        match self {
            FinalityLevel::Fast => 66.0,
            FinalityLevel::Safe => 80.0,
            FinalityLevel::UltraSafe => 90.0,
        }
    }

    pub fn from_stake_percent(stake: f64) -> Self {
        if stake >= 90.0 {
            FinalityLevel::UltraSafe
        } else if stake >= 80.0 {
            FinalityLevel::Safe
        } else {
            FinalityLevel::Fast
        }
    }
}

/// Complete verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub signature: Signature,
    pub slot: u64,
    pub verified: bool,
    pub risk_score: f64,
    pub finality_level: FinalityLevel,
    pub network_health: NetworkHealth,
    pub consensus_count: u8,
    pub timestamp: DateTime<Utc>,
}

impl VerificationResult {
    pub fn new(signature: Signature, slot: u64) -> Self {
        Self {
            signature,
            slot,
            verified: false,
            risk_score: 1.0,
            finality_level: FinalityLevel::Fast,
            network_health: NetworkHealth::Healthy,
            consensus_count: 0,
            timestamp: Utc::now(),
        }
    }

    pub fn with_verification(mut self, verified: bool) -> Self {
        self.verified = verified;
        self
    }

    pub fn with_consensus(mut self, count: u8) -> Self {
        self.consensus_count = count;
        self
    }

    pub fn with_finality(mut self, level: FinalityLevel) -> Self {
        self.finality_level = level;
        self
    }

    pub fn with_network_health(mut self, health: NetworkHealth) -> Self {
        self.network_health = health;
        self
    }

    pub fn with_risk_score(mut self, score: f64) -> Self {
        self.risk_score = score.clamp(0.0, 1.0);
        self
    }

    pub fn is_safe(&self) -> bool {
        self.verified
            && self.network_health.is_operational()
            && self.risk_score < 0.2
            && self.finality_level >= FinalityLevel::Safe
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_finality_levels() {
        assert_eq!(FinalityLevel::Fast.required_stake_percent(), 66.0);
        assert_eq!(FinalityLevel::from_stake_percent(95.0), FinalityLevel::UltraSafe);
        assert!(FinalityLevel::UltraSafe > FinalityLevel::Fast);
    }

    #[test]
    fn test_verification_result_builder() {
        let sig = Signature::from_str(
            "5VERv8NMvzbJMEkV8xnrLkEaWRtSz9CosKDYjCJjBRnbJLgp8uirBgmQpjKhoR4tjF3ZpRzrFmBV6UjKdiSZkQUW"
        ).unwrap();
        
        let result = VerificationResult::new(sig, 12345)
            .with_verification(true)
            .with_consensus(4)
            .with_finality(FinalityLevel::UltraSafe)
            .with_risk_score(0.05);

        assert!(result.is_safe());
    }
}
