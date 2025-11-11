use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use serde::Serialize;
use tokio::sync::broadcast;
use tracing::{debug, info};

use crate::types::VerificationResult;

/// WebSocket state
#[derive(Clone)]
pub struct WsState {
    pub tx: broadcast::Sender<VerificationEvent>,
}

impl WsState {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self { tx }
    }

        pub fn broadcast(&self, event: VerificationEvent) {
                let _ = self.tx.send(event);
    }
}

/// Verification event for WebSocket
#[derive(Debug, Clone, Serialize)]
pub struct VerificationEvent {
    pub signature: String,
    pub verified: bool,
    pub slot: u64,
    pub risk_score: f64,
    pub timestamp: String,
}

impl From<VerificationResult> for VerificationEvent {
    fn from(result: VerificationResult) -> Self {
        Self {
            signature: result.signature.to_string(),
            verified: result.verified,
            slot: result.slot,
            risk_score: result.risk_score,
            timestamp: result.timestamp.to_rfc3339(),
        }
    }
}

/// WebSocket upgrade handler
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<WsState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

/// Handle WebSocket connection
async fn handle_socket(socket: WebSocket, state: WsState) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.tx.subscribe();

    info!("New WebSocket connection");

    // Spawn task to send events to client
    let mut send_task = tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            let json = serde_json::to_string(&event).unwrap();
            if sender.send(Message::Text(json)).await.is_err() {
                break;
            }
        }
    });

    // Handle incoming messages (ping/pong)
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Close(_) = msg {
                break;
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }

    debug!("WebSocket connection closed");
}