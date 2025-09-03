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
    login_events.write(LoginRequested {
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
    queue_events.write(JoinQueueRequested {
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

### WebSocket Events - Sending Messages

```rust
fn trigger_websocket_actions(
    mut join_queue: EventWriter<JoinQueueRequested>,
    mut leave_queue: EventWriter<LeaveQueueRequested>,
    mut connect_websocket: EventWriter<ConnectWebSocketRequested>,
    mut send_message: EventWriter<SendWebSocketMessageRequested>,
) {
    // Connect WebSocket (usually automatic after login)
    connect_websocket.write(ConnectWebSocketRequested {
        jwt_token: "your_jwt_token".to_string(),
    });
    
    // Join matchmaking queue
    join_queue.write(JoinQueueRequested {
        game_mode: GameMode::Ranked,
    });
    // Sends: {"type": "queue_join"}
    
    // Leave matchmaking queue  
    leave_queue.write(LeaveQueueRequested);
    // Sends: {"type": "queue_leave"}
    
    // Send custom WebSocket message
    send_message.write(SendWebSocketMessageRequested {
        message: json!({"type": "custom", "data": "value"}),
    });
}
```

### WebSocket Events - Handling Responses

```rust
fn handle_websocket_responses(
    mut connection_established: EventReader<ConnectionEstablished>,
    mut queue_join_response: EventReader<QueueJoinResponse>,
    mut match_found: EventReader<MatchFound>,
) {
    for event in connection_established.read() {
        println!("Connected: {}", event.message);
    }
    
    for event in queue_join_response.read() {
        if event.success {
            println!("Queue joined: {}", event.message);
        }
    }
    
    for event in match_found.read() {
        println!("Match found! Connect to {}:{} with secret: {}", 
            event.server_host, event.server_port, event.server_secret);
        // Now connect to the game server
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