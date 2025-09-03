use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use crate::networking::{PlayerProfile, GameResult};

// UI → Network Events (Requests)
#[derive(Event)]
pub struct LoginRequested {
    pub username: String,
    pub password: String,
}

#[derive(Event)]
pub struct LogoutRequested;

#[derive(Event)]
pub struct JoinQueueRequested {
    pub game_mode: GameMode,
}

#[derive(Event)]
pub struct LeaveQueueRequested;

#[derive(Event)]
pub struct StartOfflineGameRequested {
    pub game_mode: GameMode,
    pub difficulty: Difficulty,
}

#[derive(Event)]
pub struct SyncNowRequested;

// Network → UI Events (Responses)
#[derive(Event)]
pub struct LoginCompleted {
    pub success: bool,
    pub player_profile: Option<PlayerProfile>,
    pub error: Option<String>,
}

#[derive(Event)]
pub struct LogoutCompleted;

#[derive(Event)]
pub struct MatchFound {
    pub match_id: u64,
    pub server_host: String,
    pub server_port: u16,
    pub server_secret: String,
    pub players: Vec<u64>,
}

#[derive(Event)]
pub struct QueueJoined {
    pub estimated_wait_time: Option<Duration>,
}

#[derive(Event)]
pub struct QueueLeft;

#[derive(Event)]
pub struct ConnectionEstablished;

#[derive(Event)]
pub struct ConnectionLost {
    pub reason: String,
    pub retry_in: Duration,
}

#[derive(Event)]
pub struct SyncCompleted {
    pub success: bool,
    pub conflicts: Vec<String>,
}

#[derive(Event)]
pub struct NetworkError {
    pub error: String,
    pub recoverable: bool,
}

// Supporting Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameMode {
    Ranked,
    Casual,
    Practice,
}

impl std::fmt::Display for GameMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameMode::Ranked => write!(f, "ranked"),
            GameMode::Casual => write!(f, "casual"),
            GameMode::Practice => write!(f, "practice"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

// API Request/Response Types
#[derive(Serialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<PlayerData>,
}

#[derive(Deserialize)]
pub struct PlayerData {
    pub player_id: u64,
    pub username: String,
    pub token: String,
}