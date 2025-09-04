use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::time::{Duration, SystemTime};
use tokio::sync::mpsc;

#[derive(Resource)]
pub struct NetworkManager {
    pub connection_state: ConnectionState,
    pub current_player: Option<PlayerProfile>,
    pub websocket_receiver: Option<mpsc::UnboundedReceiver<WebSocketMessage>>,
    pub websocket_sender: Option<mpsc::UnboundedSender<serde_json::Value>>,
    pub http_client: reqwest::Client,
    pub sync_queue: VecDeque<PendingSync>,
    pub base_url: String,
    pub websocket_url: String,
}

impl Default for NetworkManager {
    fn default() -> Self {
        Self {
            connection_state: ConnectionState::Offline,
            current_player: None,
            websocket_receiver: None,
            websocket_sender: None,
            http_client: reqwest::Client::new(),
            sync_queue: VecDeque::new(),
            base_url: "http://localhost:3333".to_string(),
            websocket_url: "ws://localhost:3334".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    Offline,
    Connecting,
    Online,
    Syncing,
}

#[derive(Debug, Clone)]
pub struct WebSocketMessage {
    pub message_type: String,
    pub data: serde_json::Value,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PendingSync {
    pub action: SyncAction,
    pub timestamp: SystemTime,
    pub retry_count: u32,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum SyncAction {
    UpdateProfile(PlayerProfile),
    RecordGameResult(GameResult),
    UpdateSettings(GameSettings),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlayerProfile {
    pub jwt_token: String,
    pub user_id: u64,
    pub username: String,
    pub level: u32,
    pub elo: u32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GameResult {
    pub match_id: u64,
    pub winner: Option<u64>,
    pub duration: Duration,
    pub final_score: (u32, u32),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GameSettings {
    pub audio_volume: f32,
    pub graphics_quality: String,
}

impl NetworkManager {
    pub fn is_online(&self) -> bool {
        matches!(
            self.connection_state,
            ConnectionState::Online | ConnectionState::Syncing
        )
    }

    pub fn queue_sync(&mut self, action: SyncAction) {
        let pending_sync = PendingSync {
            action,
            timestamp: SystemTime::now(),
            retry_count: 0,
        };
        self.sync_queue.push_back(pending_sync);
    }

    pub fn set_websocket_channels(
        &mut self,
        receiver: mpsc::UnboundedReceiver<WebSocketMessage>,
        sender: mpsc::UnboundedSender<serde_json::Value>,
    ) {
        self.websocket_receiver = Some(receiver);
        self.websocket_sender = Some(sender);
        self.connection_state = ConnectionState::Online;
    }

    pub fn send_websocket_message(&mut self, message: serde_json::Value) -> Result<(), String> {
        if let Some(sender) = &self.websocket_sender {
            sender
                .send(message)
                .map_err(|e| format!("Failed to send WebSocket message: {}", e))?;
            Ok(())
        } else {
            Err("WebSocket not connected".to_string())
        }
    }

    pub fn disconnect_websocket(&mut self) {
        self.websocket_receiver = None;
        self.websocket_sender = None;
        self.connection_state = ConnectionState::Offline;
    }

    pub fn set_current_player(&mut self, player_profile: PlayerProfile) {
        self.current_player = Some(player_profile);
    }

    pub fn clear_current_player(&mut self) {
        self.current_player = None;
        self.connection_state = ConnectionState::Offline;
    }

    pub fn get_player_id(&self) -> Option<u64> {
        self.current_player.as_ref().map(|p| p.user_id)
    }

    pub fn get_jwt_token(&self) -> Option<&str> {
        self.current_player.as_ref().map(|p| p.jwt_token.as_str())
    }

    pub fn is_logged_in(&self) -> bool {
        self.current_player.is_some()
    }
}
