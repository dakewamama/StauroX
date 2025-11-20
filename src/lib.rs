pub mod api;
pub mod config;
pub mod error;
pub mod monitor;
pub mod parsers;
pub mod rpc;
pub mod service;
pub mod types;
pub mod verification;

// Re-exports
pub use api::{ApiState, WsState};
pub use config::{Config, Network};
pub use error::{Result, StauroXError};
pub use parsers::{BridgeInstruction, BridgeType, ParsedTransaction, TransactionParser};  // NEW
pub use service::VerificationService;
pub use types::{FinalityLevel, NetworkHealth, SlotObservation, VerificationResult};
pub use verification::VerificationEngine;