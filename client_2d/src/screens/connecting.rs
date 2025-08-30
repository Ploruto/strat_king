//! The connecting screen for joining a game server.

use bevy::prelude::*;
use crate::{screens::Screen, theme::widget};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Connecting), spawn_connecting_ui);
    app.add_systems(Update, handle_server_connection.run_if(in_state(Screen::Connecting)));
}

#[derive(Component)]
struct ConnectionStatus {
    connection_time: f32,
    is_connecting: bool,
}

fn spawn_connecting_ui(mut commands: Commands) {
    commands.spawn((
        widget::ui_root("Connecting"),
        GlobalZIndex(1),
        StateScoped(Screen::Connecting),
        ConnectionStatus {
            connection_time: 0.0,
            is_connecting: false,
        },
        children![
            widget::header("Connecting to Game"),
            widget::label("Establishing connection to game server..."),
            widget::label("This may take a few moments"),
        ],
    ));
}

fn handle_server_connection(
    mut connection_query: Query<&mut ConnectionStatus>,
    mut next_screen: ResMut<NextState<Screen>>,
    time: Res<Time>,
) {
    for mut status in connection_query.iter_mut() {
        if !status.is_connecting {
            status.is_connecting = true;
            info!("Starting game server connection...");
            
            // TODO: This is where we'll integrate lightyear client
            // 1. Get server connection info from matchmaking result
            // 2. Create lightyear client with authentication token
            // 3. Connect to dedicated game server via UDP
        }
        
        status.connection_time += time.delta_secs();
        
        // Mock: Auto-connect after 3 seconds
        // TODO: Replace with real lightyear connection logic
        if status.connection_time > 3.0 {
            info!("Connected to game server! Starting match...");
            next_screen.set(Screen::Gameplay);
            break;
        }
    }
}