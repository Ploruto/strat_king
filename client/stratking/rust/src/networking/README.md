# Networking Module

A clean, minimal implementation of the networking architecture from SPEC.md.

## Usage

### Basic Setup
```rust
use crate::networking::NetworkingPlugin;

app.add_plugins(NetworkingPlugin);
```

### Login Example
```rust
fn handle_login_button(
    mut login_events: EventWriter<LoginRequested>,
) {
    login_events.send(LoginRequested {
        username: "player123".to_string(),
        password: "password123".to_string(),
    });
}

fn handle_login_response(
    mut login_completed: EventReader<LoginCompleted>,
) {
    for event in login_completed.read() {
        if event.success {
            println!("Logged in as: {}", event.player_profile.as_ref().unwrap().username);
        } else {
            println!("Login failed: {}", event.error.as_ref().unwrap());
        }
    }
}
```

### Matchmaking Example
```rust
fn join_ranked_queue(
    mut queue_events: EventWriter<JoinQueueRequested>,
) {
    queue_events.send(JoinQueueRequested {
        game_mode: GameMode::Ranked,
    });
}

fn handle_match_found(
    mut match_events: EventReader<MatchFound>,
) {
    for match_info in match_events.read() {
        println!("Match found! Connect to: {}:{}", 
            match_info.server_host, 
            match_info.server_port
        );
    }
}
```

### WebSocket Connection
```rust
// After successful login, establish WebSocket connection
async fn connect_websocket(jwt_token: String) {
    use crate::networking::websocket::start_websocket_connection;
    
    match start_websocket_connection(jwt_token, "ws://localhost:3333".to_string()).await {
        Ok((receiver, sender)) => {
            // Set up channels in NetworkManager
        }
        Err(e) => {
            println!("WebSocket connection failed: {}", e);
        }
    }
}
```

## Architecture

- **NetworkManager**: Resource holding connection state and WebSocket channels
- **Events**: Clean event-driven communication between UI and networking logic  
- **Systems**: Centralized processing of network requests and responses
- **WebSocket**: Background task with channel communication to stay in Bevy's main thread

## Key Features

- Offline-first design with sync queue
- Event-driven architecture
- WebSocket connection with reconnection handling
- HTTP client for authentication and API calls
- Main thread networking (Bevy/Godot compatible)