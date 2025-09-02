# Mobile Client Architecture Specification

## Overview

This document specifies the architecture for a mobile Strategy King client built with Godot (UI) and Bevy (game logic), featuring offline-first design, centralized networking, and event-driven communication.

## Core Principles

- **Offline-First**: App functions immediately without network connection
- **Event-Driven**: Clean separation between UI events and business logic
- **Main Thread Networking**: All network operations on main thread for Godot compatibility
- **Local Storage**: JWT tokens, player data, and progress stored locally
- **Sync When Available**: Background synchronization when network is available

---

## 1. Architecture Overview

### Technology Stack
- **UI Layer**: Godot 4.x (screens, input, mobile features, animations)
- **Logic Layer**: Bevy ECS (game state, networking, business logic)
- **Bridge**: Event/Resource system between Godot and Bevy
- **Storage**: Local persistence (SQLite or JSON files)

### Separation of Concerns
```
┌─────────────────────────────────────────┐
│                 GODOT                   │
│  • UI Screens & Scenes                  │
│  • Input Handling                       │
│  • Animations & Effects                 │
│  • Mobile-specific Features             │
└─────────────────┬───────────────────────┘
                  │ Events & Resources
┌─────────────────▼───────────────────────┐
│                 BEVY                    │
│  • Game Logic & State                   │
│  • Networking (HTTP/WebSocket)          │
│  • Data Persistence                     │
│  • Business Rules                       │
└─────────────────────────────────────────┘
```

---

## 2. Network Architecture

### Centralized NetworkManager

All networking happens through a single `NetworkManager` resource running on the main thread:

```rust
#[derive(Resource)]
pub struct NetworkManager {
    pub connection_state: ConnectionState,
    pub auth_client: Option<AuthClient>,
    pub websocket_client: Option<WebSocketClient>,
    pub game_client: Option<GameClient>,
    pub sync_queue: VecDeque<PendingSync>,
}

#[derive(Debug, Clone)]
pub enum ConnectionState {
    Offline,
    Connecting,
    Online,
    Syncing,
}
```

### Event-Driven Network Communication

Instead of screen-specific networking, use centralized event handling:

```rust
// Events FROM UI to NetworkManager
#[derive(Event)]
pub struct LoginRequested {
    pub username: String,
    pub password: String,
}

#[derive(Event)]
pub struct JoinQueueRequested {
    pub game_mode: GameMode,
}

#[derive(Event)]
pub struct StartOfflineGameRequested {
    pub game_mode: GameMode,
    pub difficulty: Difficulty,
}

// Events FROM NetworkManager to UI
#[derive(Event)]
pub struct LoginCompleted {
    pub success: bool,
    pub player_profile: Option<PlayerProfile>,
    pub error: Option<String>,
}

#[derive(Event)]
pub struct MatchFound {
    pub match_info: MatchInfo,
    pub server_details: ServerConnection,
}

#[derive(Event)]
pub struct ConnectionLost {
    pub reason: String,
    pub retry_in: Duration,
}
```

### Network Systems

```rust
// Main networking system that processes all events
fn process_network_events(
    mut network_manager: ResMut<NetworkManager>,
    mut login_events: EventReader<LoginRequested>,
    mut queue_events: EventReader<JoinQueueRequested>,
    mut login_completed: EventWriter<LoginCompleted>,
    mut match_found: EventWriter<MatchFound>,
) {
    // Handle all network events centrally
}

// Background sync system
fn background_sync(
    mut network_manager: ResMut<NetworkManager>,
    local_storage: Res<LocalStorage>,
    time: Res<Time>,
) {
    // Sync local changes when online
    // Handle connection recovery
}
```

---

## 3. Data Architecture (Offline-First)

### Local Storage Structure

```rust
#[derive(Resource, Serialize, Deserialize)]
pub struct LocalStorage {
    pub player_profile: Option<PlayerProfile>,
    pub game_settings: GameSettings,
    pub offline_progress: OfflineProgress,
    pub sync_queue: Vec<PendingSync>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PlayerProfile {
    pub jwt_token: String,
    pub user_id: u64,
    pub username: String,
    pub level: u32,
    pub elo: u32,
}

#[derive(Serialize, Deserialize)]
pub struct GameSettings {
    pub audio_volume: f32,
}

#[derive(Serialize, Deserialize)]
pub struct OfflineProgress {
}
```

### Sync Management

```rust
#[derive(Serialize, Deserialize)]
pub struct PendingSync {
    pub action: SyncAction,
    pub timestamp: SystemTime,
    pub retry_count: u32,
}

#[derive(Serialize, Deserialize)]
pub enum SyncAction {
    UpdateProfile(PlayerProfile),
    RecordGameResult(GameResult),
    UnlockAchievement(AchievementId),
    UpdateSettings(GameSettings),
}

// System to handle sync conflicts
fn handle_sync_conflicts(
    mut local_storage: ResMut<LocalStorage>,
    mut conflict_events: EventReader<SyncConflict>,
) {
    // Strategy: Server wins for competitive data (elo, level)
    //          Client wins for preferences (settings, progress)
}
```

---

## 4. Screen/Scene Architecture

### Screen Flow

```
App Launch
    ↓
SplashScreen (loading local data)
    ↓
MainMenuScreen ←─────────────────┐
    ↓                           │
    ├── Play → GameModeSelectionScreen
    │              ↓
    │              ├── Online → MatchmakingScreen → GameScreen
    │              └── Offline → GameScreen
    │
    ├── Profile → ProfileScreen
    │
    ├── Settings → SettingsScreen
    │
    └── [First Launch Only] → AuthScreen ───┘

GameScreen → GameResultScreen → MainMenuScreen
```

### Screen Resources (Persistent State)

```rust
// These resources persist across screen changes
#[derive(Resource)]
pub struct AuthenticationState {
    pub local_profile: Option<PlayerProfile>,
    pub sync_status: SyncStatus,
    pub requires_login: bool,
}

#[derive(Resource)]
pub struct MatchmakingState {
    pub queue_status: QueueStatus,
    pub estimated_wait_time: Option<Duration>,
    pub current_match: Option<MatchInfo>,
}

#[derive(Resource)]
pub struct GameState {
    pub current_session: Option<GameSession>,
    pub game_mode: Option<GameMode>,
    pub is_online: bool,
}

#[derive(Resource)]
pub struct ConnectionState {
    pub is_online: bool,
    pub last_sync: Option<SystemTime>,
    pub pending_syncs: u32,
}
```

### Godot Scene Structure

```
Main.tscn (root scene)
├── UI (CanvasLayer)
│   ├── SplashScreen.tscn
│   ├── MainMenuScreen.tscn
│   ├── AuthScreen.tscn (conditional)
│   ├── GameModeSelectionScreen.tscn
│   ├── MatchmakingScreen.tscn
│   ├── ProfileScreen.tscn
│   ├── SettingsScreen.tscn
│   └── GameScreen.tscn
├── Overlays (CanvasLayer)
│   ├── ConnectionStatusOverlay.tscn
│   ├── LoadingOverlay.tscn
│   ├── ErrorDialog.tscn
│   └── NotificationToast.tscn
```

---

## 5. Event System Design

### UI → Logic Events

```rust
// Authentication Events
LoginRequested { username, password }
LogoutRequested
RefreshTokenRequested

// Matchmaking Events
JoinQueueRequested { game_mode, preferences }
LeaveQueueRequested
AcceptMatchRequested { match_id }

// Game Events
StartOfflineGameRequested { mode, difficulty }
GameActionRequested { action }
SurrenderRequested
PauseGameRequested

// Settings Events
SettingsUpdateRequested { settings }
SyncNowRequested
```

### Logic → UI Events

```rust
// Authentication Events
LoginCompleted { success, profile?, error? }
TokenRefreshCompleted { success }
LogoutCompleted

// Matchmaking Events
QueueJoined { estimated_wait_time }
QueueLeft
MatchFound { match_info, server_details }
MatchCancelled { reason }

// Connection Events
ConnectionEstablished
ConnectionLost { reason, retry_in }
SyncStarted
SyncCompleted { conflicts? }

// Game Events
GameStarted { session_info }
GameEnded { results }
GameStateChanged { new_state }

// Error Events
NetworkError { error, recoverable }
ValidationError { field, message }
```

---

### Main Thread Guarantee

```rust
// Ensure all Godot interactions happen on main thread
pub fn setup_main_thread_systems(app: &mut App) {
    app
        .configure_sets(Update, MainThreadSet.run_if(on_main_thread))
        .add_systems(Update, (
            process_network_events,
            update_godot_ui,
            handle_godot_signals,
        ).in_set(MainThreadSet));
}

pub fn on_main_thread() -> bool {
    // Check if running on main thread for Godot compatibility
    std::thread::current().name() == Some("main")
}
```

---

## 6. Data Flow & API Integration

### Backend API Endpoints

**Base URL**: `http://localhost:3333` (development) / `https://api.stratking.com` (production)

#### Authentication Flow

```rust
// POST /auth/login
#[derive(Serialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Deserialize)]
struct LoginResponse {
    data: PlayerData,
    token: String,
}

#[derive(Deserialize)]
struct PlayerData {
    player_id: u64,
    username: String,
    level: u32,
    elo: u32,
}
```

**Implementation Example**:
```rust
async fn handle_login_request(
    login_event: LoginRequested,
    mut network_manager: ResMut<NetworkManager>,
) {
    let request = LoginRequest {
        username: login_event.username,
        password: login_event.password,
    };
    
    let response = network_manager.http_client
        .post("http://localhost:3333/auth/login")
        .json(&request)
        .send()
        .await;
        
    match response {
        Ok(resp) => {
            let login_data: LoginResponse = resp.json().await.unwrap();
            // Store JWT token and player data locally
            // Emit LoginCompleted event
        }
        Err(e) => {
            // Emit LoginCompleted with error
        }
    }
}
```

### WebSocket Integration

#### Connection Setup

```rust
// WebSocket URL: ws://localhost:3333/matchmaking
// Authentication: Send JWT token on connection

async fn setup_websocket_connection(jwt_token: &str) -> Result<WebSocketStream, Error> {
    let mut request = tungstenite::connect("ws://localhost:3333/matchmaking").0;
    
    // Send authentication message first
    let auth_msg = json!({
        "type": "auth",
        "token": jwt_token
    });
    
    request.write_message(Message::Text(auth_msg.to_string())).await?;
    Ok(request)
}
```

#### WebSocket Message Types

**Outgoing Messages (Client → Server)**:

```rust
// Join matchmaking queue
{
    "type": "join_queue",
    "player_id": "1",
    "game_mode": "ranked"
}

// Leave matchmaking queue  
{
    "type": "leave_queue",
    "player_id": "1"
}
```

**Incoming Messages (Server → Client)**:

```rust
// Queue joined successfully
{
    "type": "queue_joined",
    "estimated_wait_time": 30
}

// Match found
{
    "type": "match_found",
    "data": {
        "matchId": 17,
        "players": [1, 2],
        "status": "spawning",
        "serverHost": "127.0.0.1",
        "serverPort": 32770,
        "serverSecret": "abc123",
        "message": "Game server is starting..."
    }
}

// Queue left
{
    "type": "queue_left"
}

// Error
{
    "type": "error",
    "message": "Authentication failed"
}
```

**WebSocket Handler Implementation**:

```rust
async fn handle_websocket_messages(
    mut websocket: WebSocketStream,
    mut match_events: EventWriter<MatchFound>,
    mut queue_events: EventWriter<QueueStatusChanged>,
) {
    while let Some(message) = websocket.next().await {
        match message {
            Ok(Message::Text(text)) => {
                let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();
                
                match parsed["type"].as_str() {
                    Some("match_found") => {
                        let data = &parsed["data"];
                        match_events.send(MatchFound {
                            match_id: data["matchId"].as_u64().unwrap(),
                            server_host: data["serverHost"].as_str().unwrap().to_string(),
                            server_port: data["serverPort"].as_u64().unwrap() as u16,
                            server_secret: data["serverSecret"].as_str().unwrap().to_string(),
                            players: data["players"].as_array().unwrap()
                                .iter()
                                .map(|p| p.as_u64().unwrap())
                                .collect(),
                        });
                    }
                    Some("queue_joined") => {
                        queue_events.send(QueueStatusChanged {
                            status: QueueStatus::InQueue,
                            estimated_wait: parsed["estimated_wait_time"].as_u64().map(Duration::from_secs),
                        });
                    }
                    Some("error") => {
                        // Handle WebSocket errors
                    }
                    _ => warn!("Unknown WebSocket message type: {}", text),
                }
            }
            Ok(Message::Close(_)) => break,
            Err(e) => {
                error!("WebSocket error: {}", e);
                // Implement reconnection logic
                break;
            }
            _ => {}
        }
    }
}
```

### Lightyear Game Client Integration

#### Client Setup (when match found)

```rust
use lightyear::prelude::*;
use lightyear::netcode::{Key, NetcodeConfig};

async fn create_lightyear_client(
    match_info: MatchFound,
    player_id: u64,
    mut commands: Commands,
) {
    // Parse server address
    let server_addr = SocketAddr::new(
        match_info.server_host.parse().unwrap(),
        match_info.server_port,
    );
    
    // Create unique client port (base 9000 + player_id)
    let client_port = (9000 + player_id) as u16;
    let client_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), client_port);
    
    // Create authentication for Lightyear
    let auth = Authentication::Manual {
        server_addr,
        client_id: player_id,
        private_key: Key::default(), // Use same key as server
        protocol_id: 0, // Must match server protocol_id
    };
    
    // Spawn Lightyear client entity
    let client_entity = commands.spawn((
        Name::from("GameClient"),
        Client::default(),
        LocalAddr(client_addr),
        PeerAddr(server_addr),
        Link::new(None),
        ReplicationReceiver::default(),
        NetcodeClient::new(auth, NetcodeConfig::default()).unwrap(),
        UdpIo::default(),
    )).id();
    
    // Trigger connection
    commands.trigger_targets(Connect, client_entity);
    
    info!("Lightyear client connecting: {} -> {}", client_addr, server_addr);
}
```

#### Game State Synchronization

```rust
// Example game state that gets replicated
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct PlayerPosition {
    pub x: f32,
    pub y: f32,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct GameUnit {
    pub unit_type: UnitType,
    pub health: u32,
    pub owner: u64, // player_id
}

// Register components for replication
fn setup_lightyear_protocol(app: &mut App) {
    app.add_plugins(ClientPlugin::default())
        .add_systems(Update, (
            handle_server_messages,
            send_player_actions,
            update_game_state,
        ));
        
    // Register replicated components
    app.register_component::<PlayerPosition>()
        .register_component::<GameUnit>();
}

// Handle incoming game messages from server
fn handle_server_messages(
    mut client: ResMut<Client>,
    mut game_state: ResMut<GameState>,
) {
    while let Some(message) = client.receive_message::<GameMessage>() {
        match message {
            GameMessage::GameStart { players } => {
                game_state.status = GameStatus::Playing;
                game_state.players = players;
            }
            GameMessage::UnitSpawned { unit } => {
                // Spawn unit in local game world
            }
            GameMessage::PlayerAction { player_id, action } => {
                // Apply action to game state
            }
        }
    }
}

// Send player actions to server
fn send_player_actions(
    mut client: ResMut<Client>,
    mut action_events: EventReader<PlayerActionRequested>,
) {
    for action in action_events.iter() {
        client.send_message(GameMessage::PlayerAction {
            player_id: action.player_id,
            action: action.action.clone(),
        });
    }
}
```

### Complete Flow Example

#### 1. App Launch Flow

```rust
// 1. Load local data on app start
fn load_local_data(
    mut commands: Commands,
    mut local_storage: ResMut<LocalStorage>,
) {
    if let Some(profile) = local_storage.load_player_profile() {
        commands.insert_resource(AuthenticationState {
            local_profile: Some(profile),
            sync_status: SyncStatus::Pending,
            requires_login: false,
        });
    } else {
        commands.insert_resource(AuthenticationState {
            local_profile: None,
            sync_status: SyncStatus::NotRequired,
            requires_login: true,
        });
    }
}

// 2. Background sync if we have stored JWT
fn background_sync_on_startup(
    mut network_manager: ResMut<NetworkManager>,
    auth_state: Res<AuthenticationState>,
) {
    if let Some(profile) = &auth_state.local_profile {
        // Validate JWT token
        // Sync any pending local changes
        // Update player data from server
    }
}
```

#### 2. Matchmaking Flow

```rust
// User clicks "Play" -> GameModeSelection -> "Ranked Match"
// Godot emits signal: join_queue_requested("ranked")

fn handle_join_queue(
    mut queue_events: EventReader<JoinQueueRequested>,
    mut network_manager: ResMut<NetworkManager>,
    auth_state: Res<AuthenticationState>,
) {
    for event in queue_events.iter() {
        if let Some(profile) = &auth_state.local_profile {
            // Send WebSocket message to join queue
            let message = json!({
                "type": "join_queue",
                "player_id": profile.user_id.to_string(),
                "game_mode": event.game_mode.to_string()
            });
            
            network_manager.websocket_send(message);
        }
    }
}

// Server responds with match_found -> Create Lightyear client -> Connect to game server
```

#### 3. Game Session Flow

```rust
// 1. Match found -> Create Lightyear client
// 2. Connect to game server
// 3. Receive initial game state
// 4. User interactions -> Game events -> Lightyear messages
// 5. Server updates -> State replication -> UI updates
// 6. Game ends -> Results -> Return to main menu

fn handle_game_end(
    mut game_events: EventReader<GameEnded>,
    mut network_manager: ResMut<NetworkManager>,
    mut local_storage: ResMut<LocalStorage>,
) {
    for event in game_events.iter() {
        // Update local stats
        local_storage.update_game_stats(&event.results);
        
        // Queue sync for when online
        if !network_manager.is_online() {
            local_storage.queue_sync(SyncAction::RecordGameResult(event.results.clone()));
        } else {
            // Immediate sync
            network_manager.sync_game_results(&event.results);
        }
        
        // Transition back to main menu
        // Emit GameSessionEnded event for Godot UI
    }
}
```

### Error Handling Patterns

```rust
// Network request with retry logic
async fn make_request_with_retry<T>(
    url: &str,
    body: &impl Serialize,
    max_retries: u32,
) -> Result<T, NetworkError> 
where 
    T: for<'de> Deserialize<'de>,
{
    let mut attempts = 0;
    let mut delay = Duration::from_secs(1);
    
    loop {
        match reqwest::Client::new()
            .post(url)
            .json(body)
            .send()
            .await 
        {
            Ok(response) => {
                if response.status().is_success() {
                    return Ok(response.json().await.unwrap());
                } else if attempts < max_retries {
                    attempts += 1;
                    tokio::time::sleep(delay).await;
                    delay *= 2; // Exponential backoff
                } else {
                    return Err(NetworkError::RequestFailed);
                }
            }
            Err(_) if attempts < max_retries => {
                attempts += 1;
                tokio::time::sleep(delay).await;
                delay *= 2;
            }
            Err(e) => return Err(NetworkError::ConnectionFailed(e.to_string())),
        }
    }
}

// WebSocket reconnection logic
async fn maintain_websocket_connection(
    mut network_manager: ResMut<NetworkManager>,
    mut connection_events: EventWriter<ConnectionLost>,
) {
    let mut reconnect_attempts = 0;
    let max_attempts = 5;
    
    while reconnect_attempts < max_attempts {
        match network_manager.connect_websocket().await {
            Ok(stream) => {
                reconnect_attempts = 0;
                network_manager.websocket = Some(stream);
                break;
            }
            Err(e) => {
                reconnect_attempts += 1;
                let delay = Duration::from_secs(2_u64.pow(reconnect_attempts));
                
                connection_events.send(ConnectionLost {
                    reason: e.to_string(),
                    retry_in: delay,
                });
                
                tokio::time::sleep(delay).await;
            }
        }
    }
}
```
