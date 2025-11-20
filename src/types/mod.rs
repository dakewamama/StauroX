pub mod network;
pub mod verification;

// Re-export commonly used types
pub use network::{NetworkHealth, SlotObservation};
pub use verification::{FinalityLevel, VerificationResult};
