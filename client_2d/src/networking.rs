//! Networking module for authentication and matchmaking.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<NetworkingState>();
}

/// Global networking state resource
#[derive(Resource, Default)]
pub struct NetworkingState {
    pub is_connected: bool,
    pub auth_token: Option<String>,
    pub player_id: Option<String>,
    pub server_url: String,
}

impl NetworkingState {
    pub fn new() -> Self {
        Self {
            is_connected: false,
            auth_token: None,
            player_id: None,
            server_url: "http://localhost:3000".to_string(), // Backend service URL
        }
    }
}

/// Authentication request payload
#[derive(Serialize)]
pub struct AuthRequest {
    pub username: String,
    pub password: String,
}

/// Authentication response
#[derive(Deserialize)]
pub struct AuthResponse {
    pub success: bool,
    pub token: Option<String>,
    pub player_id: Option<String>,
    pub message: String,
}

/// Queue join request
#[derive(Serialize)]
pub struct QueueRequest {
    pub game_mode: String,
    pub player_id: String,
}

/// Match found message (from WebSocket)
#[derive(Deserialize)]
pub struct MatchFoundMessage {
    pub match_id: String,
    pub server_host: String,
    pub server_port: u16,
    pub auth_token: String,
    pub team: u8,
}

/// WebSocket message types
#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum WebSocketMessage {
    #[serde(rename = "queue_status")]
    QueueStatus { position: u32, estimated_wait: u32 },
    #[serde(rename = "match_found")]
    MatchFound(MatchFoundMessage),
    #[serde(rename = "queue_cancelled")]
    QueueCancelled { reason: String },
}

// TODO: Implement actual HTTP client functions
// These would be async functions that make requests to the backend service

pub async fn authenticate_user(username: &str, password: &str) -> Result<AuthResponse, String> {
    // TODO: Replace with actual HTTP request
    // let response = reqwest::post(&format!("{}/auth/login", server_url))
    //     .json(&AuthRequest { username, password })
    //     .send()
    //     .await?;

    // Mock response for now
    // tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

    Ok(AuthResponse {
        success: true,
        token: Some("mock_jwt_token".to_string()),
        player_id: Some("player_123".to_string()),
        message: "Authentication successful".to_string(),
    })
}

pub async fn join_queue(game_mode: &str, player_id: &str) -> Result<(), String> {
    // TODO: Replace with actual HTTP request to join queue
    // let response = reqwest::post(&format!("{}/matchmaking/join", server_url))
    //     .json(&QueueRequest { game_mode, player_id })
    //     .send()
    //     .await?;

    // Mock success
    // tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    Ok(())
}

// TODO: Implement WebSocket connection handling
// pub async fn connect_websocket(auth_token: &str) -> Result<WebSocketStream, String> {
//     let url = format!("ws://localhost:3000/matchmaking/ws?token={}", auth_token);
//     let (ws_stream, _) = connect_async(url).await.map_err(|e| e.to_string())?;
//     Ok(ws_stream)
// }
