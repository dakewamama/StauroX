use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

use crate::error::{Result, StauroXError};

/// Generic consensus engine - works with any type
pub struct ConsensusEngine {
    threshold: usize,
    total_sources: usize,
}

impl ConsensusEngine {
    pub fn new(threshold: usize, total_sources: usize) -> Self {
        Self {
            threshold,
            total_sources,
        }
    }

    /// Get consensus threshold
    pub fn threshold(&self) -> usize {
        self.threshold
    }

    /// Check if we have enough responses
    pub fn has_minimum_responses<T>(&self, responses: &[T]) -> Result<()> {
        if responses.len() < self.threshold {
            return Err(StauroXError::consensus_failure(
                responses.len(),
                self.threshold,
            ));
        }
        Ok(())
    }

    /// Find consensus value from responses (simple majority)
    pub fn find_consensus<T>(&self, responses: Vec<T>) -> Result<T>
    where
        T: Eq + Hash + Clone + Debug,
    {
        self.has_minimum_responses(&responses)?;

        let mut counts: HashMap<T, usize> = HashMap::new();
        for response in &responses {
            *counts.entry(response.clone()).or_insert(0) += 1;
        }

        // Find the most common value
        let (consensus_value, count) = counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .ok_or_else(|| StauroXError::consensus_failure(0, self.threshold))?;

        // Verify it meets threshold
        if count < self.threshold {
            return Err(StauroXError::consensus_failure(count, self.threshold));
        }

        Ok(consensus_value)
    }

    /// Calculate consensus ratio (for metrics)
    pub fn consensus_ratio<T>(&self, responses: &[T]) -> f64
    where
        T: Eq + Hash,
    {
        if responses.is_empty() {
            return 0.0;
        }

        let mut counts: HashMap<&T, usize> = HashMap::new();
        for response in responses {
            *counts.entry(response).or_insert(0) += 1;
        }

        let max_count = *counts.values().max().unwrap_or(&0);
        max_count as f64 / responses.len() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_consensus_success() {
        let engine = ConsensusEngine::new(3, 4);
        let responses = vec![100u64, 100, 100, 101];
        
        let result = engine.find_consensus(responses);
        assert_eq!(result.unwrap(), 100);
    }

    #[test]
    fn test_find_consensus_failure() {
        let engine = ConsensusEngine::new(3, 4);
        let responses = vec![100u64, 101, 102, 103];
        
        let result = engine.find_consensus(responses);
        assert!(result.is_err());
    }

    #[test]
    fn test_insufficient_responses() {
        let engine = ConsensusEngine::new(3, 4);
        let responses = vec![100u64, 100];
        
        let result = engine.find_consensus(responses);
        assert!(result.is_err());
    }

    #[test]
    fn test_consensus_ratio() {
        let engine = ConsensusEngine::new(3, 4);
        let responses = vec![100u64, 100, 100, 101];
        
        let ratio = engine.consensus_ratio(&responses);
        assert_eq!(ratio, 0.75);
    }
}