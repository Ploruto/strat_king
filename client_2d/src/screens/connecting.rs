//! The connecting screen for joining a game server.

use crate::{networking::NetworkingState, screens::Screen, theme::widget};
use bevy::prelude::*;
use core::net::{IpAddr, Ipv4Addr, SocketAddr};
use lightyear::netcode::Key;
use lightyear::prelude::client::*;
use lightyear::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Connecting), spawn_connecting_ui);
    app.add_systems(
        Update,
        handle_server_connection.run_if(in_state(Screen::Connecting)),
    );
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
    networking_state: Res<NetworkingState>,
    mut commands: Commands,
    time: Res<Time>,
) {
    for mut status in connection_query.iter_mut() {
        if !status.is_connecting {
            status.is_connecting = true;
            info!("Starting game server connection...");

            // Get server connection info from matchmaking result
            if let Some(game_server) = &networking_state.game_server {
                info!(
                    "Connecting to game server: {}:{}",
                    game_server.server_host, game_server.server_port
                );

                // Parse the server address
                let server_ip: IpAddr = game_server
                    .server_host
                    .parse()
                    .unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST));
                let server_addr = SocketAddr::new(server_ip, game_server.server_port);

                // Create a unique client address using player ID to avoid port conflicts
                let client_id = networking_state.player_id
                    .as_ref()
                    .and_then(|id| id.parse::<u64>().ok())
                    .unwrap_or(1);
                
                // Use player ID to create unique client port (base port 9000 + player_id)
                let client_port = (9000 + client_id) as u16;
                let client_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), client_port);

                // Create authentication using the server secret
                let auth = Authentication::Manual {
                    server_addr,
                    client_id,
                    private_key: Key::default(),
                    protocol_id: 0,
                };

                info!(
                    "Creating Lightyear client: {} -> {}",
                    client_addr, server_addr
                );

                // Spawn the Lightyear client
                let client = commands
                    .spawn((
                        Client::default(),
                        LocalAddr(client_addr),
                        PeerAddr(server_addr),
                        Link::new(None),
                        ReplicationReceiver::default(),
                        NetcodeClient::new(auth, NetcodeConfig::default()).unwrap(),
                        UdpIo::default(),
                    ))
                    .id();

                // Trigger the connection
                commands.trigger_targets(Connect, client);

                println!("---- Lightyear client created and connection initiated ----");

                // TODO: Monitor connection status and transition to gameplay when connected
                // For now, wait a bit and transition to gameplay
                status.connection_time = 0.0;
            } else {
                error!("No game server info available!");
                next_screen.set(Screen::Title);
            }
        } else {
            // Monitor connection progress
            status.connection_time += time.delta_secs();

            // After 3 seconds, assume connection is successful and transition
            // TODO: Replace with actual connection status monitoring
            if status.connection_time > 3.0 {
                info!("Transitioning to gameplay screen");
                next_screen.set(Screen::Gameplay);
            }
        }
    }
}
