//! The matchmaking screen for finding online matches.

use crate::{screens::Screen, theme::widget};
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Matchmaking), spawn_matchmaking_ui);
    app.add_systems(
        Update,
        (handle_queue_join, handle_matchmaking_updates).run_if(in_state(Screen::Matchmaking)),
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
            (
                QueueButton,
                widget::button("Join 1v1 Queue", join_queue),
            ),
            (
                QueueStatus,
                widget::label("Not in queue"),
            ),
            widget::button("Back to Main Menu", back_to_main_menu),
        ],
    ));
}

fn join_queue(_: Trigger<Pointer<Click>>, mut matchmaking_query: Query<&mut MatchmakingStatus>) {
    for mut status in matchmaking_query.iter_mut() {
        if !status.in_queue {
            status.in_queue = true;
            status.queue_time = 0.0;
            info!("Joining 1v1 queue...");

            // TODO: Send HTTP request to backend to join queue
            // TODO: Establish WebSocket connection for real-time updates

            // Mock: simulate websocket connection
            status.websocket_connected = true;
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

            // Mock: simulate match found after 5 seconds
            // TODO: Replace with real WebSocket message handling
            if status.queue_time > 5.0 {
                info!("Match found! Connecting to game server...");

                // TODO: Receive game server connection info from WebSocket
                // TODO: Create lightyear client connection

                next_screen.set(Screen::Connecting);
                break;
            }
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
