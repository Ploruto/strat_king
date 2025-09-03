// Example usage of WebSocket events - This file is for documentation only

use bevy::prelude::*;
use serde_json::json;
use crate::networking::events::*;

/// Example system showing how to trigger WebSocket messages
fn example_websocket_usage(
    // Event writers for triggering WebSocket actions
    mut join_queue_events: EventWriter<JoinQueueRequested>,
    mut leave_queue_events: EventWriter<LeaveQueueRequested>,
    mut connect_websocket_events: EventWriter<ConnectWebSocketRequested>,
    mut disconnect_websocket_events: EventWriter<DisconnectWebSocketRequested>,
    mut send_message_events: EventWriter<SendWebSocketMessageRequested>,
    
    // Event readers for handling responses
    mut connection_established: EventReader<ConnectionEstablished>,
    mut queue_join_response: EventReader<QueueJoinResponse>,
    mut match_found: EventReader<MatchFound>,
) {
    // 1. MANUALLY Connect WebSocket (usually automatic after login)
    // connect_websocket_events.write(ConnectWebSocketRequested {
    //     jwt_token: "your_jwt_token_here".to_string(),
    // });

    // 2. Join matchmaking queue
    join_queue_events.write(JoinQueueRequested {
        game_mode: GameMode::Ranked,
    });
    // This sends: {"type": "queue_join"}

    // 3. Leave matchmaking queue
    leave_queue_events.write(LeaveQueueRequested);
    // This sends: {"type": "queue_leave"}

    // 4. Send custom WebSocket message
    send_message_events.write(SendWebSocketMessageRequested {
        message: json!({
            "type": "custom_message",
            "data": {
                "some_field": "some_value"
            }
        }),
    });

    // 5. Disconnect WebSocket
    disconnect_websocket_events.write(DisconnectWebSocketRequested);

    // Handle incoming WebSocket responses
    for event in connection_established.read() {
        info!("WebSocket connected: {}", event.message);
    }

    for event in queue_join_response.read() {
        if event.success {
            info!("Successfully joined queue: {}", event.message);
        } else {
            warn!("Failed to join queue: {}", event.message);
        }
    }

    for event in match_found.read() {
        info!(
            "Match found! Match ID: {}, Server: {}:{}, Secret: {}",
            event.match_id, event.server_host, event.server_port, event.server_secret
        );
        // Connect to game server using the provided details
    }
}