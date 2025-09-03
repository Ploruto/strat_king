use crate::networking::events::*;
use crate::networking::{ConnectionState, NetworkManager, PlayerProfile};
use bevy::prelude::*;
use bevy_tokio_tasks::TokioTasksRuntime;
use serde_json::json;
use std::time::Duration;

pub fn http_system(
    mut network_manager: ResMut<NetworkManager>,
    mut login_events: EventReader<LoginRequested>,
    mut logout_events: EventReader<LogoutRequested>,
    mut sync_events: EventReader<SyncNowRequested>,
    mut login_completed: EventWriter<LoginCompleted>,
    mut logout_completed: EventWriter<LogoutCompleted>,
    mut sync_completed: EventWriter<SyncCompleted>,
    mut network_errors: EventWriter<NetworkError>,
    runtime: ResMut<TokioTasksRuntime>,
) {
    // Handle login requests
    for login_event in login_events.read() {
        let request = LoginRequest {
            username: login_event.username.clone(),
            password: login_event.password.clone(),
        };

        network_manager.connection_state = ConnectionState::Connecting;

        // Spawn async task using bevy-tokio-tasks
        runtime.spawn_background_task({
            let client = network_manager.http_client.clone();
            let base_url = network_manager.base_url.clone();
            let username = login_event.username.clone();
            let password = login_event.password.clone();

            |mut ctx| async move {
                let request = LoginRequest {
                    username: username.clone(),
                    password: password.clone(),
                };

                let login_url = format!("{}/auth/login", base_url);
                let response = client
                    .post(&login_url)
                    .json(&request)
                    .timeout(Duration::from_secs(10))
                    .send()
                    .await;

                match response {
                    Ok(resp) => {
                        match resp.json::<LoginResponse>().await {
                            Ok(login_response) => {
                                if login_response.success {
                                    if let Some(data) = login_response.data {
                                        let profile = PlayerProfile {
                                            jwt_token: data.token,
                                            user_id: data.player_id,
                                            username: data.username,
                                            level: 1, // Default level since not provided by backend
                                            elo: 1000, // Default elo since not provided by backend
                                        };

                                        info!("Login successful for user: {}", profile.username);

                                        // Send success event back to main thread
                                        ctx.run_on_main_thread(move |ctx| {
                                            let mut login_completed =
                                                ctx.world.resource_mut::<Events<LoginCompleted>>();
                                            login_completed.send(LoginCompleted {
                                                success: true,
                                                player_profile: Some(profile),
                                                error: None,
                                            });
                                        })
                                        .await;
                                    } else {
                                        // Success=true but no data - shouldn't happen
                                        ctx.run_on_main_thread(move |ctx| {
                                            let mut login_completed =
                                                ctx.world.resource_mut::<Events<LoginCompleted>>();
                                            login_completed.send(LoginCompleted {
                                                success: false,
                                                player_profile: None,
                                                error: Some(
                                                    "Login response missing data".to_string(),
                                                ),
                                            });
                                        })
                                        .await;
                                    }
                                } else {
                                    // Backend returned success=false
                                    error!("Login failed: {}", login_response.message);

                                    ctx.run_on_main_thread(move |ctx| {
                                        let mut login_completed =
                                            ctx.world.resource_mut::<Events<LoginCompleted>>();
                                        login_completed.send(LoginCompleted {
                                            success: false,
                                            player_profile: None,
                                            error: Some(login_response.message),
                                        });
                                    })
                                    .await;
                                }
                            }
                            Err(e) => {
                                error!("Failed to parse login response: {}", e);

                                ctx.run_on_main_thread(move |ctx| {
                                    let mut login_completed =
                                        ctx.world.resource_mut::<Events<LoginCompleted>>();
                                    login_completed.send(LoginCompleted {
                                        success: false,
                                        player_profile: None,
                                        error: Some(format!("Failed to parse response: {}", e)),
                                    });
                                })
                                .await;
                            }
                        }
                    }
                    Err(e) => {
                        error!("Login request failed: {}", e);

                        ctx.run_on_main_thread(move |ctx| {
                            let mut login_completed =
                                ctx.world.resource_mut::<Events<LoginCompleted>>();
                            login_completed.send(LoginCompleted {
                                success: false,
                                player_profile: None,
                                error: Some(format!("Network error: {}", e)),
                            });
                        })
                        .await;
                    }
                }
            }
        });
    }

    // Handle logout requests
    for _logout_event in logout_events.read() {
        network_manager.clear_current_player();
        network_manager.disconnect_websocket();
        logout_completed.write(LogoutCompleted);
    }

    // Handle sync requests
    for _sync_event in sync_events.read() {
        if !network_manager.is_online() {
            network_errors.write(NetworkError {
                error: "Cannot sync while offline".to_string(),
                recoverable: true,
            });
            continue;
        }

        network_manager.connection_state = ConnectionState::Syncing;

        // Process sync queue
        let mut conflicts = Vec::new();

        // In a real implementation, you'd process each pending sync
        while let Some(_pending) = network_manager.sync_queue.pop_front() {
            // Make HTTP request to sync the data
            // Handle conflicts and retries
        }

        network_manager.connection_state = ConnectionState::Online;
        sync_completed.write(SyncCompleted {
            success: true,
            conflicts,
        });
    }
}

pub fn login_success_system(
    mut network_manager: ResMut<NetworkManager>,
    mut login_completed: EventReader<LoginCompleted>,
    mut connect_websocket: EventWriter<ConnectWebSocketRequested>,
) {
    for login_event in login_completed.read() {
        if login_event.success {
            if let Some(profile) = &login_event.player_profile {
                network_manager.set_current_player(profile.clone());
                info!(
                    "Player profile stored in NetworkManager: {}",
                    profile.username
                );

                // Automatically connect WebSocket after successful login
                connect_websocket.write(ConnectWebSocketRequested {
                    jwt_token: profile.jwt_token.clone(),
                });
            }
        } else {
            // Login failed, clear any existing player data
            network_manager.clear_current_player();
        }
    }
}

pub fn websocket_connection_system(
    mut network_manager: ResMut<NetworkManager>,
    mut connect_events: EventReader<ConnectWebSocketRequested>,
    mut disconnect_events: EventReader<DisconnectWebSocketRequested>,
    mut send_message_events: EventReader<SendWebSocketMessageRequested>,
    mut network_errors: EventWriter<NetworkError>,
    runtime: ResMut<TokioTasksRuntime>,
) {
    // Handle WebSocket connection requests
    for connect_event in connect_events.read() {
        if network_manager.websocket_receiver.is_some() {
            info!("WebSocket already connected, ignoring connection request");
            continue;
        }

        network_manager.connection_state = ConnectionState::Connecting;

        // Spawn async task to connect WebSocket
        runtime.spawn_background_task({
            let jwt_token = connect_event.jwt_token.clone();
            let websocket_url = network_manager.websocket_url.clone();

            |mut ctx| async move {
                match crate::networking::websocket::start_websocket_connection(
                    jwt_token,
                    websocket_url,
                )
                .await
                {
                    Ok((receiver, sender)) => {
                        info!("WebSocket connection established");

                        ctx.run_on_main_thread(move |ctx| {
                            if let Some(mut network_manager) =
                                ctx.world.get_resource_mut::<NetworkManager>()
                            {
                                network_manager.set_websocket_channels(receiver, sender);
                            }
                        })
                        .await;
                    }
                    Err(e) => {
                        error!("Failed to connect WebSocket: {}", e);

                        ctx.run_on_main_thread(move |ctx| {
                            if let Some(mut network_manager) =
                                ctx.world.get_resource_mut::<NetworkManager>()
                            {
                                network_manager.connection_state = ConnectionState::Offline;
                            }

                            let mut network_errors =
                                ctx.world.resource_mut::<Events<NetworkError>>();
                            network_errors.send(NetworkError {
                                error: format!("WebSocket connection failed: {}", e),
                                recoverable: true,
                            });
                        })
                        .await;
                    }
                }
            }
        });
    }

    // Handle WebSocket disconnection requests
    for _disconnect_event in disconnect_events.read() {
        network_manager.disconnect_websocket();
        info!("WebSocket disconnected");
    }

    // Handle custom WebSocket message requests
    for send_event in send_message_events.read() {
        if let Err(e) = network_manager.send_websocket_message(send_event.message.clone()) {
            network_errors.write(NetworkError {
                error: format!("Failed to send WebSocket message: {}", e),
                recoverable: true,
            });
        }
    }
}

pub fn queue_system(
    mut network_manager: ResMut<NetworkManager>,
    mut join_queue_events: EventReader<JoinQueueRequested>,
    mut leave_queue_events: EventReader<LeaveQueueRequested>,
    mut network_errors: EventWriter<NetworkError>,
) {
    // Handle join queue requests
    for join_event in join_queue_events.read() {
        if !network_manager.is_online() {
            network_errors.write(NetworkError {
                error: "Cannot join queue while offline".to_string(),
                recoverable: true,
            });
            continue;
        }

        let message = json!({
            "type": "queue_join"
        });

        if let Err(e) = network_manager.send_websocket_message(message) {
            network_errors.write(NetworkError {
                error: format!("Failed to join queue: {}", e),
                recoverable: true,
            });
        }
    }

    // Handle leave queue requests
    for _leave_event in leave_queue_events.read() {
        if !network_manager.is_online() {
            continue;
        }

        let message = json!({
            "type": "queue_leave"
        });

        if let Err(e) = network_manager.send_websocket_message(message) {
            network_errors.write(NetworkError {
                error: format!("Failed to leave queue: {}", e),
                recoverable: true,
            });
        }
    }
}
