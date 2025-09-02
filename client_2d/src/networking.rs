//! Networking module for authentication and matchmaking.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Event fired when a match is found via WebSocket
#[derive(Event)]
pub struct MatchFoundEvent {
    pub match_id: i64,
    pub server_host: String,
    pub server_port: u16,
    pub server_secret: String,
}

/// Resource to communicate match found status from WebSocket task
#[derive(Resource, Default)]
pub struct MatchFoundStatus {
    pub pending_match: Option<GameServerInfo>,
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<NetworkingState>();
    app.init_resource::<MatchFoundStatus>();
    app.add_event::<MatchFoundEvent>();
}

/// Global networking state resource
#[derive(Resource)]
pub struct NetworkingState {
    pub is_connected: bool,
    pub auth_token: Option<String>,
    pub player_id: Option<String>,
    pub server_url: String,
    pub game_server: Option<GameServerInfo>,
}

#[derive(Debug, Clone)]
pub struct GameServerInfo {
    pub match_id: i64,
    pub server_host: String,
    pub server_port: u16,
    pub server_secret: String,
}

impl NetworkingState {
    pub fn new() -> Self {
        Self {
            is_connected: false,
            auth_token: None,
            player_id: None,
            server_url: "http://localhost:3333".to_string(), // Backend service URL (matches our AdonisJS backend)
            game_server: None,
        }
    }
}

impl Default for NetworkingState {
    fn default() -> Self {
        Self::new()
    }
}

/// Authentication request payload
#[derive(Serialize, Debug)]
pub struct AuthRequest {
    pub username: String,
    pub password: String,
}

/// Authentication response (matches backend structure)
#[derive(Deserialize, Debug)]
pub struct AuthResponse {
    pub success: bool,
    pub message: String,
    pub data: AuthData,
}

#[derive(Deserialize, Debug)]
pub struct AuthData {
    pub player_id: u32,
    pub username: String,
    pub token: String,
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

pub async fn authenticate_user(
    username: &str,
    password: &str,
    server_url: &str,
) -> Result<AuthResponse, String> {
    let client = reqwest::Client::new();

    let auth_request = AuthRequest {
        username: username.to_string(),
        password: password.to_string(),
    };

    let url = format!("{}/auth/login", server_url);
    println!("Sending request to: {}", url);
    println!("Request payload: {:?}", auth_request);

    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&auth_request)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.status().is_success() {
        let auth_response: AuthResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok(auth_response)
    } else {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());

        Err(format!("Authentication failed: {}", error_text))
    }
}

pub async fn join_queue(
    game_mode: &str,
    player_id: &str,
    server_url: &str,
    auth_token: &str,
) -> Result<(), String> {
    let client = reqwest::Client::new();

    let queue_request = QueueRequest {
        game_mode: game_mode.to_string(),
        player_id: player_id.to_string(),
    };

    println!("Join q with token: ${auth_token}");

    let response = client
        .post(&format!("{}/matchmaking/join", server_url))
        .header("Authorization", &format!("Bearer {}", auth_token))
        .header("Content-Type", "application/json")
        .json(&queue_request)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.status().is_success() {
        Ok(())
    } else {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());

        Err(format!("Queue join failed: {}", error_text))
    }
}

pub async fn connect_websocket(
    auth_token: &str,
    server_url: &str,
) -> Result<
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    String,
> {
    let ws_url = server_url.replace("http://localhost:3333", "ws://localhost:3334");
    let url = format!("{}/ws?token={}", ws_url, auth_token);

    println!("Connecting to WebSocket: {}", url);

    let (ws_stream, _response) = tokio_tungstenite::connect_async(&url)
        .await
        .map_err(|e| format!("WebSocket connection failed: {}", e))?;

    Ok(ws_stream)
}

pub async fn listen_for_messages(
    mut ws_stream: tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    mut ctx: bevy_tokio_tasks::TaskContext,
) {
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::tungstenite::Message;

    while let Some(message) = ws_stream.next().await {
        match message {
            Ok(Message::Text(text)) => {
                println!("WebSocket message received: {}", text);

                // Parse the message using dynamic JSON parsing
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) {
                    if let Some(msg_type) = parsed["type"].as_str() {
                        match msg_type {
                            "connection_success" => {
                                if let Some(message) = parsed["data"]["message"].as_str() {
                                    info!("WebSocket connected: {}", message);
                                }
                            }
                            "match_found" => {
                                if let Some(data) = parsed["data"].as_object() {
                                    if let (
                                        Some(match_id),
                                        Some(server_host),
                                        Some(server_port),
                                        Some(server_secret),
                                    ) = (
                                        data["matchId"].as_i64(),
                                        data["serverHost"].as_str(),
                                        data["serverPort"].as_u64(),
                                        data["serverSecret"].as_str(),
                                    ) {
                                        info!(
                                            "Match found! Match ID: {}, Server: {}:{}, Secret: {}",
                                            match_id, server_host, server_port, server_secret
                                        );

                                        // Update MatchFoundStatus resource on main thread
                                        let match_info = GameServerInfo {
                                            match_id,
                                            server_host: server_host.to_string(),
                                            server_port: server_port as u16,
                                            server_secret: server_secret.to_string(),
                                        };
                                        
                                        ctx.run_on_main_thread(move |ctx| {
                                            if let Some(mut match_status) = ctx.world.get_resource_mut::<MatchFoundStatus>() {
                                                match_status.pending_match = Some(match_info);
                                                info!("Updated MatchFoundStatus resource with server info");
                                            }
                                        }).await;
                                    } else {
                                        warn!("Match found message missing required fields");
                                    }
                                }
                            }
                            "queue_status" => {
                                if let Some(data) = parsed["data"].as_object() {
                                    if let (Some(position), Some(wait)) =
                                        (data["position"].as_i64(), data["estimated_wait"].as_i64())
                                    {
                                        info!(
                                            "Queue position: {}, estimated wait: {}s",
                                            position, wait
                                        );
                                    }
                                }
                            }
                            "queue_cancelled" => {
                                if let Some(reason) = parsed["data"]["reason"].as_str() {
                                    info!("Queue cancelled: {}", reason);
                                }
                            }
                            _ => {
                                info!("Unknown WebSocket message type: {}", msg_type);
                            }
                        }
                    }
                } else {
                    warn!("Failed to parse WebSocket message: {}", text);
                }
            }
            Ok(Message::Close(_)) => {
                info!("WebSocket connection closed");
                break;
            }
            Err(e) => {
                error!("WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }
}
