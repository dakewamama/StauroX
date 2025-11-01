use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::error::{Result, StauroXError};

/// Solana network type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Network {
    Mainnet,
    Devnet,
}

impl Network {
    pub fn default_endpoints(&self) -> Vec<String> {
        match self {
            Network::Mainnet => vec![
                "https://api.mainnet-beta.solana.com".to_string(),
                "https://rpc.ankr.com/solana".to_string(),
                "https://solana-api.projectserum.com".to_string(),
                "https://solana.publicnode.com".to_string(),
            ],
            Network::Devnet => vec![
                "https://api.devnet.solana.com".to_string(),
                "https://rpc-devnet.helius.xyz".to_string(),
                "https://devnet.rpcpool.com".to_string(),
                "https://api.devnet.solana.com".to_string(),
            ],
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Network::Mainnet => "mainnet-beta",
            Network::Devnet => "devnet",
        }
    }
}

/// StauroX configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub network: Network,
    pub rpc: RpcConfig,
    pub monitoring: MonitoringConfig,
    pub api: ApiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcConfig {
    pub endpoints: Vec<String>,
    pub consensus_threshold: usize,
    pub request_timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub health_check_interval_ms: u64,
    pub slot_retention_seconds: u64,
    pub stale_threshold_seconds: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub websocket_port: u16,
    pub rest_port: u16,
}

impl Config {
    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        if self.rpc.endpoints.is_empty() {
            return Err(StauroXError::config("At least one RPC endpoint required"));
        }

        if self.rpc.consensus_threshold == 0 {
            return Err(StauroXError::config("Consensus threshold must be > 0"));
        }

        if self.rpc.consensus_threshold > self.rpc.endpoints.len() {
            return Err(StauroXError::config(format!(
                "Consensus threshold ({}) exceeds number of endpoints ({})",
                self.rpc.consensus_threshold,
                self.rpc.endpoints.len()
            )));
        }

        if self.monitoring.health_check_interval_ms == 0 {
            return Err(StauroXError::config(
                "Health check interval must be > 0",
            ));
        }

        Ok(())
    }

    pub fn health_check_interval(&self) -> Duration {
        Duration::from_millis(self.monitoring.health_check_interval_ms)
    }

    pub fn request_timeout(&self) -> Duration {
        Duration::from_millis(self.rpc.request_timeout_ms)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::devnet()
    }
}

impl Config {
    /// Create devnet configuration (safer for testing)
    pub fn devnet() -> Self {
        let network = Network::Devnet;
        Self {
            network,
            rpc: RpcConfig {
                endpoints: network.default_endpoints(),
                consensus_threshold: 3,
                request_timeout_ms: 5000,
            },
            monitoring: MonitoringConfig {
                health_check_interval_ms: 400,
                slot_retention_seconds: 30,
                stale_threshold_seconds: 5,
            },
            api: ApiConfig {
                websocket_port: 8080,
                rest_port: 8081,
            },
        }
    }

    /// Create mainnet configuration (production)
    pub fn mainnet() -> Self {
        let network = Network::Mainnet;
        Self {
            network,
            rpc: RpcConfig {
                endpoints: network.default_endpoints(),
                consensus_threshold: 3,
                request_timeout_ms: 5000,
            },
            monitoring: MonitoringConfig {
                health_check_interval_ms: 400,
                slot_retention_seconds: 30,
                stale_threshold_seconds: 5,
            },
            api: ApiConfig {
                websocket_port: 8080,
                rest_port: 8081,
            },
        }
    }

    /// Create custom configuration with specific endpoints
    pub fn custom(network: Network, endpoints: Vec<String>) -> Self {
        let mut config = match network {
            Network::Mainnet => Self::mainnet(),
            Network::Devnet => Self::devnet(),
        };
        config.rpc.endpoints = endpoints;
        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_is_valid() {
        let config = Config::default();
        assert!(config.validate().is_ok());
        assert_eq!(config.network, Network::Devnet);
    }

    #[test]
    fn test_devnet_config() {
        let config = Config::devnet();
        assert!(config.validate().is_ok());
        assert_eq!(config.network, Network::Devnet);
        assert_eq!(config.rpc.endpoints.len(), 4);
    }

    #[test]
    fn test_mainnet_config() {
        let config = Config::mainnet();
        assert!(config.validate().is_ok());
        assert_eq!(config.network, Network::Mainnet);
        assert_eq!(config.rpc.endpoints.len(), 4);
    }

    #[test]
    fn test_custom_config() {
        let custom_endpoints = vec!["https://custom.rpc".to_string()];
        let config = Config::custom(Network::Devnet, custom_endpoints.clone());
        assert!(config.validate().is_ok());
        assert_eq!(config.rpc.endpoints, custom_endpoints);
    }

    #[test]
    fn test_network_endpoints() {
        let devnet_endpoints = Network::Devnet.default_endpoints();
        let mainnet_endpoints = Network::Mainnet.default_endpoints();
        
        assert!(!devnet_endpoints.is_empty());
        assert!(!mainnet_endpoints.is_empty());
        assert_ne!(devnet_endpoints, mainnet_endpoints);
    }

    #[test]
    fn test_invalid_consensus_threshold() {
        let mut config = Config::default();
        config.rpc.consensus_threshold = 10;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_empty_endpoints() {
        let mut config = Config::default();
        config.rpc.endpoints.clear();
        assert!(config.validate().is_err());
    }
}