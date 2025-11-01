use std::sync::Arc;
use tracing::Level;
use tracing_subscriber;

use staurox::{Config, Network, VerificationService};

#[tokio::main]
async fn main() -> staurox::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .init();

    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    let network = if args.len() > 1 && args[1] == "mainnet" {
        Network::Mainnet
    } else {
        Network::Devnet // Default to devnet for safety
    };

    // Load configuration for selected network
    let config = match network {
        Network::Mainnet => {
            tracing::warn!("Running on MAINNET - this is PRODUCTION!");
            Config::mainnet()
        }
        Network::Devnet => {
            tracing::info!("Running on DEVNET - safe for testing");
            Config::devnet()
        }
    };

    // Create and run service
    let service = Arc::new(VerificationService::new(config)?);
    service.run().await
}