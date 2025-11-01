pub mod config;
pub mod error;
pub mod monitor;
pub mod rpc;
pub mod service;
pub mod types;

// Re-export main types for convenience
pub use config::Config;
pub use error::{Result, StauroXError};
pub use service::VerificationService;
pub use types::{FinalityLevel, NetworkHealth, SlotObservation, VerificationResult};
