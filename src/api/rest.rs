use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use solana_sdk::signature::Signature;
use std::sync::Arc;
use tracing::info;

use crate::error::StauroXError;
use crate::types::VerificationResult;
use crate::verification::VerificationEngine;

/// API State
#[derive(Clone)]
pub struct ApiState {
    pub engine: Arc<VerificationEngine>,
}

/// Request body for verification
#[derive(Debug, Deserialize)]
pub struct VerifyRequest {
    pub signature: String,
}

/// Response for verification
#[derive(Debug, Serialize)]
pub struct VerifyResponse {
    pub signature: String,
    pub verified: bool,
    pub slot: u64,
    pub risk_score: f64,
    pub finality_level: String,
    pub network_health: String,
    pub timestamp: String,
}

impl From<VerificationResult> for VerifyResponse {
    fn from(result: VerificationResult) -> Self {
        Self {
            signature: result.signature.to_string(),
            verified: result.verified,
            slot: result.slot,
            risk_score: result.risk_score,
            finality_level: format!("{:?}", result.finality_level),
            network_health: format!("{:?}", result.network_health),
            timestamp: result.timestamp.to_rfc3339(),
        }
    }
}

/// Health check response
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub network: String,
}

/// Create REST API router
pub fn create_router(state: ApiState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/verify", post(verify_transaction))
        .route("/verify/:signature", get(get_verification))
        .with_state(state)
}

/// Health check endpoint
async fn health_check(State(state): State<ApiState>) -> impl IntoResponse {
    let network_health = state.engine.health_monitor.get_health().await;
    
    Json(HealthResponse {
        status: "ok".to_string(),
        network: format!("{:?}", network_health),  // Changed from network_health
    })
}

/// Verify transaction endpoint
async fn verify_transaction(
    State(state): State<ApiState>,
    Json(payload): Json<VerifyRequest>,
) -> Result<Json<VerifyResponse>, AppError> {
    info!("API: Verifying transaction {}", payload.signature);

    let signature = payload
        .signature
        .parse::<Signature>()
        .map_err(|_| AppError::InvalidSignature)?;

    let result = state
        .engine
        .verify_transaction(&signature)
        .await
        .map_err(AppError::Verification)?;

    Ok(Json(result.into()))
}

/// Get verification status endpoint
async fn get_verification(
    State(state): State<ApiState>,
    Path(signature): Path<String>,
) -> Result<Json<VerifyResponse>, AppError> {
    info!("API: Getting verification for {}", signature);

    let signature = signature
        .parse::<Signature>()
        .map_err(|_| AppError::InvalidSignature)?;

    let result = state
        .engine
        .verify_transaction(&signature)
        .await
        .map_err(AppError::Verification)?;

    Ok(Json(result.into()))
}

/// API error wrapper
pub enum AppError {
    InvalidSignature,
    Verification(StauroXError),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            AppError::InvalidSignature => (
                StatusCode::BAD_REQUEST,
                "Invalid transaction signature".to_string(),
            ),
            AppError::Verification(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Verification error: {}", e),
            ),
        };

        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}