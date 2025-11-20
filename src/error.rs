use thiserror::Error;

/// All possible errors in the StauroX system
#[derive(Debug, Error)]
pub enum StauroXError {
    #[error("RPC error: {0}")]
    Rpc(#[from] solana_client::client_error::ClientError),

    #[error("Consensus failed: {message}")]
    ConsensusFailure {
        message: String,
        responses: usize,
        required: usize,
    },

    #[error("Network health check failed: {0}")]
    HealthCheck(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Signature parse error: {0}")]
    SignatureParse(#[from] solana_sdk::signature::ParseSignatureError),

    #[error("Transaction verification failed: {0}")]
    Verification(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, StauroXError>;

impl StauroXError {
    pub fn consensus_failure(responses: usize, required: usize) -> Self {
        Self::ConsensusFailure {
            message: format!(
                "Insufficient consensus: {}/{} responses",
                responses, required
            ),
            responses,
            required,
        }
    }

    pub fn health_check(msg: impl Into<String>) -> Self {
        Self::HealthCheck(msg.into())
    }

    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }

    pub fn verification(msg: impl Into<String>) -> Self {
        Self::Verification(msg.into())
    }
}