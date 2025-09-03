use crate::networking::events::*;
use crate::networking::{ConnectionState, NetworkManager, WebSocketMessage};
use bevy::prelude::*;
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

pub async fn start_websocket_connection(
    jwt_token: String,
    websocket_url: String,
) -> Result<
    (
        mpsc::UnboundedReceiver<WebSocketMessage>,
        mpsc::UnboundedSender<Value>,
    ),
    Box<dyn std::error::Error + Send + Sync>,
> {
    let ws_url = format!("{}/ws?token={}", websocket_url, jwt_token);

    let (ws_stream, _) = connect_async(&ws_url).await?;
    let (mut ws_sink, mut ws_stream) = ws_stream.split();

    // Create channels for communication with Bevy
    let (message_tx, message_rx) = mpsc::unbounded_channel::<WebSocketMessage>();
    let (command_tx, mut command_rx) = mpsc::unbounded_channel::<Value>();

    // Spawn task to handle incoming messages
    let incoming_tx = message_tx.clone();
    tokio::spawn(async move {
        while let Some(message) = ws_stream.next().await {
            match message {
                Ok(Message::Text(text)) => {
                    if let Ok(parsed) = serde_json::from_str::<Value>(&text) {
                        if let Some(msg_type) = parsed.get("type").and_then(|v| v.as_str()) {
                            let ws_message = WebSocketMessage {
                                message_type: msg_type.to_string(),
                                data: parsed.clone(),
                            };

                            if incoming_tx.send(ws_message).is_err() {
                                warn!(
                                    "Failed to send WebSocket message to Bevy - receiver dropped"
                                );
                                break;
                            }
                        }
                    }
                }
                Ok(Message::Close(_)) => {
                    info!("WebSocket connection closed by server");
                    break;
                }
                Err(e) => {
                    error!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }
    });

    // Spawn task to handle outgoing messages
    tokio::spawn(async move {
        while let Some(command) = command_rx.recv().await {
            if let Err(e) = ws_sink
                .send(Message::Text(command.to_string().into()))
                .await
            {
                error!("Failed to send WebSocket message: {}", e);
                break;
            }
        }
    });

    Ok((message_rx, command_tx))
}

pub fn websocket_system(
    mut network_manager: ResMut<NetworkManager>,
    mut match_found_events: EventWriter<MatchFound>,
    mut queue_joined_events: EventWriter<QueueJoined>,
    mut queue_join_response_events: EventWriter<QueueJoinResponse>,
    mut queue_left_events: EventWriter<QueueLeft>,
    mut connection_established_events: EventWriter<ConnectionEstablished>,
    mut connection_lost_events: EventWriter<ConnectionLost>,
) {
    if let Some(receiver) = &mut network_manager.websocket_receiver {
        // Process all available messages without blocking
        while let Ok(message) = receiver.try_recv() {
            match message.message_type.as_str() {
                "connection_success" => {
                    if let Some(data) = message.data.get("data") {
                        if let Some(message_text) = data.get("message").and_then(|v| v.as_str()) {
                            connection_established_events.write(ConnectionEstablished {
                                message: message_text.to_string(),
                            });
                            info!("WebSocket connected: {}", message_text);
                        }
                    }
                }
                "queue_join_response" => {
                    if let Some(data) = message.data.get("data") {
                        let success = data
                            .get("success")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);
                        let message_text = data
                            .get("message")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Unknown response");

                        queue_join_response_events.write(QueueJoinResponse {
                            success,
                            message: message_text.to_string(),
                        });

                        if success {
                            info!("Queue join successful: {}", message_text);
                        } else {
                            warn!("Queue join failed: {}", message_text);
                        }
                    }
                }
                "match_found" => {
                    if let Some(data) = message.data.get("data") {
                        if let (
                            Some(match_id),
                            Some(server_host),
                            Some(server_port),
                            Some(server_secret),
                            Some(players),
                        ) = (
                            data.get("matchId").and_then(|v| v.as_u64()),
                            data.get("serverHost").and_then(|v| v.as_str()),
                            data.get("serverPort").and_then(|v| v.as_u64()),
                            data.get("serverSecret").and_then(|v| v.as_str()),
                            data.get("players").and_then(|v| v.as_array()),
                        ) {
                            let player_ids: Vec<u64> =
                                players.iter().filter_map(|p| p.as_u64()).collect();

                            match_found_events.write(MatchFound {
                                match_id,
                                server_host: server_host.to_string(),
                                server_port: server_port as u16,
                                server_secret: server_secret.to_string(),
                                players: player_ids,
                            });
                        }
                    }
                }
                "queue_joined" => {
                    let estimated_wait = message
                        .data
                        .get("estimated_wait_time")
                        .and_then(|v| v.as_u64())
                        .map(Duration::from_secs);

                    queue_joined_events.write(QueueJoined {
                        estimated_wait_time: estimated_wait,
                    });
                }
                "queue_left" => {
                    queue_left_events.write(QueueLeft);
                }
                "error" => {
                    if let Some(error_msg) = message.data.get("message").and_then(|v| v.as_str()) {
                        connection_lost_events.write(ConnectionLost {
                            reason: error_msg.to_string(),
                            retry_in: Duration::from_secs(5),
                        });
                    }
                }
                _ => {
                    warn!("Unknown WebSocket message type: {}", message.message_type);
                }
            }
        }
    }
}
