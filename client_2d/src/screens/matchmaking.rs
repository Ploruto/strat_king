//! The matchmaking screen for finding online matches.

use crate::{
    networking::{self, MatchFoundEvent, MatchFoundStatus, NetworkingState},
    screens::Screen,
    theme::widget,
};
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Matchmaking), spawn_matchmaking_ui);
    app.add_systems(
        Update,
        (
            handle_queue_join,
            handle_matchmaking_updates,
            handle_match_found,
            poll_match_status,
        )
            .run_if(in_state(Screen::Matchmaking)),
    );
}

#[derive(Component)]
struct MatchmakingStatus {
    in_queue: bool,
    queue_time: f32,
    websocket_connected: bool,
}

#[derive(Component)]
struct QueueButton;

#[derive(Component)]
struct QueueStatus;

fn spawn_matchmaking_ui(mut commands: Commands) {
    commands.spawn((
        widget::ui_root("Matchmaking"),
        GlobalZIndex(1),
        StateScoped(Screen::Matchmaking),
        MatchmakingStatus {
            in_queue: false,
            queue_time: 0.0,
            websocket_connected: false,
        },
        children![
            widget::header("Find Match"),
            widget::label("Select your game mode:"),
            (QueueButton, widget::button("Join 1v1 Queue", join_queue),),
            (QueueStatus, widget::label("Not in queue"),),
            widget::button("Back to Main Menu", back_to_main_menu),
        ],
    ));
}

fn join_queue(
    _: Trigger<Pointer<Click>>,
    mut matchmaking_query: Query<&mut MatchmakingStatus>,
    networking_state: Res<NetworkingState>,
    runtime: Res<bevy_tokio_tasks::TokioTasksRuntime>,
) {
    for mut status in matchmaking_query.iter_mut() {
        if !status.in_queue {
            status.in_queue = true;
            status.queue_time = 0.0;
            info!("Joining 1v1 queue...");

            // Get authentication data
            if let (Some(auth_token), Some(player_id)) =
                (&networking_state.auth_token, &networking_state.player_id)
            {
                let auth_token = auth_token.clone();
                let player_id = player_id.clone();
                let server_url = networking_state.server_url.clone();

                // First establish WebSocket connection, then join queue
                runtime.spawn_background_task(|mut ctx| async move {
                    // Connect to WebSocket first
                    match networking::connect_websocket(&auth_token, &server_url).await {
                        Ok(ws_stream) => {
                            info!("WebSocket connected successfully");

                            // Pass context to WebSocket listener for resource updates
                            let ws_ctx = ctx.clone();
                            let ws_listener = tokio::spawn(async move {
                                networking::listen_for_messages(ws_stream, ws_ctx).await;
                            });

                            // Now join the queue
                            match networking::join_queue(
                                "1v1",
                                &player_id,
                                &server_url,
                                &auth_token,
                            )
                            .await
                            {
                                Ok(()) => {
                                    info!(
                                        "Successfully joined queue - listening for match updates"
                                    );
                                }
                                Err(error) => {
                                    error!("Failed to join queue: {}", error);
                                    ws_listener.abort(); // Stop listening if queue join fails
                                }
                            }
                        }
                        Err(error) => {
                            error!("Failed to connect WebSocket: {}", error);
                        }
                    }
                });
            }
        }
    }
}

fn handle_queue_join(
    mut commands: Commands,
    matchmaking_query: Query<&MatchmakingStatus, Changed<MatchmakingStatus>>,
    button_query: Query<Entity, With<QueueButton>>,
) {
    for status in matchmaking_query.iter() {
        if status.in_queue {
            // Update button to show cancel option
            for entity in button_query.iter() {
                commands.entity(entity).despawn_recursive();
            }

            commands.spawn((
                QueueButton,
                widget::button("Cancel Queue", cancel_queue),
                StateScoped(Screen::Matchmaking),
            ));
        }
    }
}

fn handle_matchmaking_updates(
    mut matchmaking_query: Query<&mut MatchmakingStatus>,
    mut status_text_query: Query<&mut Text, With<QueueStatus>>,
    mut next_screen: ResMut<NextState<Screen>>,
    time: Res<Time>,
) {
    for mut status in matchmaking_query.iter_mut() {
        if status.in_queue {
            status.queue_time += time.delta_secs();

            // Update status text
            for mut text in status_text_query.iter_mut() {
                text.0 = format!("In queue... {:.1}s", status.queue_time);
            }

            // Real match found handling will be done via WebSocket messages
            // No more automatic scene switching
        }
    }
}

fn cancel_queue(
    _: Trigger<Pointer<Click>>,
    mut commands: Commands,
    mut matchmaking_query: Query<&mut MatchmakingStatus>,
    button_query: Query<Entity, With<QueueButton>>,
) {
    for mut status in matchmaking_query.iter_mut() {
        status.in_queue = false;
        status.websocket_connected = false;
        info!("Cancelled queue");

        // Restore join queue button
        for entity in button_query.iter() {
            commands.entity(entity).despawn_recursive();
        }

        commands.spawn((
            QueueButton,
            widget::button("Join 1v1 Queue", join_queue),
            StateScoped(Screen::Matchmaking),
        ));
    }
}

fn back_to_main_menu(_: Trigger<Pointer<Click>>, mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Title);
}

fn handle_match_found(
    mut match_events: EventReader<MatchFoundEvent>,
    mut networking_state: ResMut<NetworkingState>,
    mut next_screen: ResMut<NextState<Screen>>,
) {
    for event in match_events.read() {
        info!("Match found event received! Transitioning to connecting screen");

        // Store game server info
        networking_state.game_server = Some(crate::networking::GameServerInfo {
            match_id: event.match_id,
            server_host: event.server_host.clone(),
            server_port: event.server_port,
            server_secret: event.server_secret.clone(),
        });

        // Transition to connecting screen
        next_screen.set(Screen::Connecting);
    }
}

fn poll_match_status(
    mut match_status: ResMut<MatchFoundStatus>,
    mut networking_state: ResMut<NetworkingState>,
    mut next_screen: ResMut<NextState<Screen>>,
) {
    if let Some(match_info) = match_status.pending_match.take() {
        info!("Match found via polling! Transitioning to connecting screen");

        // Store game server info
        networking_state.game_server = Some(match_info);

        // Transition to connecting screen
        next_screen.set(Screen::Connecting);
    }
}
