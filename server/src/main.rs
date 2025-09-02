use bevy::asset::AssetPlugin;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use lightyear::prelude::server::*;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};
use shared::*;
use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::{Arc, Mutex};

use crate::map_init::{CurrentMap, MapInitPlugin};
use shared::gameplay::map::MapDefinition;

mod map_init;

#[derive(Resource, Clone)]
pub struct ServerConfig {
    pub server_secret: String,
    pub match_id: u32,
    pub expected_players: Vec<u32>,
    pub server_port: u16,
    pub server_addr: SocketAddr,
    pub backend_url: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GameState {
    WaitingForPlayers,
    MatchStarting,
    InProgress,
    Completed,
}

#[derive(Resource)]
pub struct GameStateManager {
    pub state: Arc<Mutex<GameState>>,
    pub connected_players: Arc<Mutex<Vec<u32>>>,
}

#[derive(Serialize, Deserialize)]
struct ServerReadyWebhook {
    match_id: u32,
}

#[derive(Serialize, Deserialize)]
struct MatchCompleteWebhook {
    match_id: u32,
    winner: Option<u32>,
}

fn main() -> anyhow::Result<()> {
    // Parse environment variables to match backend expectations
    let server_port = env::var("SERVER_PORT")
        .unwrap_or_else(|_| "7777".to_string())
        .parse::<u16>()
        .expect("Invalid SERVER_PORT");

    let server_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), server_port);

    let expected_players_str =
        env::var("EXPECTED_PLAYERS").expect("EXPECTED_PLAYERS environment variable is required");
    let expected_players: Vec<u32> = serde_json::from_str(&expected_players_str)
        .expect("EXPECTED_PLAYERS must be a valid JSON array of player IDs");

    let server_config = ServerConfig {
        server_secret: env::var("SERVER_SECRET").expect("SERVER_SECRET is required"),
        match_id: env::var("MATCH_ID")
            .expect("MATCH_ID is required")
            .parse()
            .expect("MATCH_ID must be a valid number"),
        expected_players: expected_players.clone(),
        server_port,
        server_addr,
        backend_url: env::var("BACKEND_URL")
            .unwrap_or_else(|_| "http://host.docker.internal:3333".to_string()),
    };

    println!("🚀 Starting Strategy King Game Server");
    println!("Match ID: {}", server_config.match_id);
    println!("Server listening on: {}", server_config.server_addr);
    println!("Expected players: {:?}", server_config.expected_players);
    println!("Backend URL: {}", server_config.backend_url);

    // Initialize game state manager
    let game_state_manager = GameStateManager {
        state: Arc::new(Mutex::new(GameState::WaitingForPlayers)),
        connected_players: Arc::new(Mutex::new(Vec::new())),
    };

    // Notify backend that server is ready (spawn blocking task)
    let config_clone = server_config.clone();
    std::thread::spawn(move || {
        // Use blocking HTTP client instead of async
        let client = reqwest::blocking::Client::new();
        let webhook_data = serde_json::json!({
            "match_id": config_clone.match_id
        });

        match client
            .post(&format!(
                "{}/webhooks/server-ready",
                config_clone.backend_url
            ))
            .json(&webhook_data)
            .send()
        {
            Ok(response) => {
                if response.status().is_success() {
                    println!("✅ Successfully notified backend that server is ready");
                } else {
                    eprintln!("⚠️ Failed to notify backend: {}", response.status());
                }
            }
            Err(e) => eprintln!("Failed to notify backend: {}", e),
        }
    });

    // Run Bevy on main thread
    let mut app = App::new();

    app.insert_resource(server_config);
    app.insert_resource(game_state_manager);
    /* MinimalPlugins:
    bevy_app:::TaskPoolPlugin,
        bevy_diagnostic:::FrameCountPlugin,
        bevy_time:::TimePlugin,
        bevy_app:::ScheduleRunnerPlugin,
        #[cfg(feature = "bevy_ci_testing")]
        bevy_dev_tools::ci_testing:::CiTestingPlugin,
        */

    /* DefaultPlugins:
    bevy_app:::PanicHandlerPlugin,
        #[cfg(feature = "bevy_log")]
        bevy_log:::LogPlugin,
        bevy_app:::TaskPoolPlugin,
        bevy_diagnostic:::FrameCountPlugin,
        bevy_time:::TimePlugin,
        bevy_transform:::TransformPlugin,
        bevy_diagnostic:::DiagnosticsPlugin,
        bevy_input:::InputPlugin,
        #[custom(cfg(not(feature = "bevy_window")))]
        bevy_app:::ScheduleRunnerPlugin,
        #[cfg(feature = "bevy_window")]
        bevy_window:::WindowPlugin,
        #[cfg(feature = "bevy_window")]
        bevy_a11y:::AccessibilityPlugin,
        #[cfg(feature = "std")]
        #[custom(cfg(any(unix, windows)))]
        bevy_app:::TerminalCtrlCHandlerPlugin,
        #[cfg(feature = "bevy_asset")]
        bevy_asset:::AssetPlugin,
        #[cfg(feature = "bevy_scene")]
        bevy_scene:::ScenePlugin,
        #[cfg(feature = "bevy_winit")]
        bevy_winit:::WinitPlugin,
        #[cfg(feature = "bevy_render")]
        bevy_render:::RenderPlugin,
        // NOTE: Load this after renderer initialization so that it knows about the supported
        // compressed texture formats.
        #[cfg(feature = "bevy_render")]
        bevy_render::texture:::ImagePlugin,
        #[cfg(feature = "bevy_render")]
        #[custom(cfg(all(not(target_arch = "wasm32"), feature = "multi_threaded")))]
        bevy_render::pipelined_rendering:::PipelinedRenderingPlugin,
        #[cfg(feature = "bevy_core_pipeline")]
        bevy_core_pipeline:::CorePipelinePlugin,
        #[cfg(feature = "bevy_sprite")]
        bevy_sprite:::SpritePlugin,
        #[cfg(feature = "bevy_text")]
        bevy_text:::TextPlugin,
        #[cfg(feature = "bevy_ui")]
        bevy_ui:::UiPlugin,
        #[cfg(feature = "bevy_pbr")]
        bevy_pbr:::PbrPlugin,
        // NOTE: Load this after renderer initialization so that it knows about the supported
        // compressed texture formats.
        #[cfg(feature = "bevy_gltf")]
        bevy_gltf:::GltfPlugin,
        #[cfg(feature = "bevy_audio")]
        bevy_audio:::AudioPlugin,
        #[cfg(feature = "bevy_gilrs")]
        bevy_gilrs:::GilrsPlugin,
        #[cfg(feature = "bevy_animation")]
        bevy_animation:::AnimationPlugin,
        #[cfg(feature = "bevy_gizmos")]
        bevy_gizmos:::GizmoPlugin,
        #[cfg(feature = "bevy_state")]
        bevy_state::app:::StatesPlugin,
        #[cfg(feature = "bevy_dev_tools")]
        bevy_dev_tools:::DevToolsPlugin,
        #[cfg(feature = "bevy_ci_testing")]
        bevy_dev_tools::ci_testing:::CiTestingPlugin,
        #[plugin_group]
        #[cfg(feature = "bevy_picking")]
        bevy_picking:::DefaultPickingPlugins,
        #[doc(hidden)]
        :IgnoreAmbiguitiesPlugin,
        */

    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin::default());
    app.add_plugins(ServerPlugins {
        tick_duration: core::time::Duration::from_secs_f64(1.0 / FIXED_TIMESTEP_HZ),
    });

    app.add_plugins(SharedPlugin);
    app.add_plugins(ServerPlugin);

    app.run();

    println!("✅ Server shutdown complete");
    Ok(())
}

// These structs are no longer needed since we use serde_json::json! macro
// but keeping them for reference if needed later

pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<MapDefinition>();
        app.add_plugins(RonAssetPlugin::<MapDefinition>::new(&[".ron"]));
        app.add_systems(Startup, (startup, load_map_asset));
        app.add_systems(Update, start_server);
        app.add_observer(handle_new_client);
        app.add_observer(handle_client_disconnect);
        app.add_systems(
            Update,
            (
                handle_ping_message,
                check_all_players_connected,
                game_state_manager,
            ),
        );
        app.add_plugins(MapInitPlugin);
    }
}

fn handle_new_client(
    trigger: Trigger<OnAdd, Connected>,
    mut commands: Commands,
    config: Res<ServerConfig>,
    game_state: Res<GameStateManager>,
) {
    let client_id = trigger.target();
    info!("🔌 New client connected: {:?}", client_id);

    // TODO: In a real implementation, validate the client's connection
    // using SERVER_SECRET and expected player ID
    // For now, we'll assume valid connections

    commands.entity(client_id).insert((ReplicationSender::new(
        SERVER_REPLICATION_INTERVAL,
        SendUpdatesMode::SinceLastAck,
        false,
    ),));

    // Add connected player to our tracking
    let game_state_clone = game_state.connected_players.clone();
    let expected_players = config.expected_players.clone();

    if let Ok(mut connected) = game_state_clone.lock() {
        let con_len = connected.len();
        // For now, just track connection count - in real implementation
        // you'd extract player ID from connection authentication
        if con_len < expected_players.len() {
            connected.push(expected_players[con_len]);
            println!(
                "👤 Player connected. Total: {}/{}",
                connected.len(),
                expected_players.len()
            );
        }
    }
}

fn handle_client_disconnect(
    trigger: Trigger<OnRemove, Connected>,
    game_state: Res<GameStateManager>,
) {
    let client_id = trigger.target();
    info!("🔌 Client disconnected: {:?}", client_id);

    if let Ok(mut connected) = game_state.connected_players.lock() {
        if !connected.is_empty() {
            connected.pop();
            println!("👤 Player disconnected. Remaining: {}", connected.len());
        }
    }
}

// Game state management systems
fn check_all_players_connected(
    config: Res<ServerConfig>,
    game_state: Res<GameStateManager>,
    mut sender: ServerMultiMessageSender,
    server: Query<&Server>,
) {
    let expected_count = config.expected_players.len();

    if let (Ok(current_state), Ok(connected)) =
        (game_state.state.lock(), game_state.connected_players.lock())
    {
        if *current_state == GameState::WaitingForPlayers && connected.len() == expected_count {
            drop(current_state);
            drop(connected);

            // Transition to MatchStarting
            if let Ok(mut state) = game_state.state.lock() {
                *state = GameState::MatchStarting;
                println!("🎯 All players connected! Starting match...");

                // TODO: Send MatchStart message via Lightyear to all connected clients
            }
        }
    }
}

fn game_state_manager(
    game_state: Res<GameStateManager>,
    mut sender: ServerMultiMessageSender,
    server: Query<&Server>,
) {
    if let Ok(current_state) = game_state.state.lock() {
        match *current_state {
            GameState::MatchStarting => {
                // Send MatchStart message to all players
                println!("📢 Broadcasting MatchStart to all players");
                // TODO: Implement actual MatchStart message sending

                // Transition to InProgress
                drop(current_state);
                if let Ok(mut state) = game_state.state.lock() {
                    *state = GameState::InProgress;
                    println!("🎮 Match is now in progress!");
                }
            }
            GameState::InProgress => {
                // Game logic runs here - handled by other systems
            }
            GameState::Completed => {
                // Match completed - handled by other systems
            }
            _ => {}
        }
    }
}

fn startup(mut commands: Commands, config: Res<ServerConfig>) {
    info!("Setting up server on {:?}", config.server_addr);
    commands.spawn((
        Name::from("GameServer"),
        Server::default(),  // ← Add Server marker component
        NetcodeServer::new(NetcodeConfig {
            protocol_id: 0,  // ← Match client protocol
            ..Default::default()
        }),
        LocalAddr(config.server_addr),
        ServerUdpIo::default(),
    ));
}

fn start_server(mut commands: Commands, server_query: Query<Entity, (With<Server>, Without<Started>)>) {
    for server_entity in server_query.iter() {
        info!("Starting server entity: {:?}", server_entity);
        commands.trigger_targets(Start, server_entity);
    }
}

fn load_map_asset(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("Loading map asset...");
    let map_handle: Handle<MapDefinition> = asset_server.load("maps/simple_1v1.map.ron");
    commands.insert_resource(CurrentMap(map_handle));
}

fn handle_ping_message(
    mut receiver: Query<&mut MessageReceiver<PingMessage>>,
    mut sender: ServerMultiMessageSender,
    server: Query<&Server>,
) {
    if let Ok(server) = server.single() {
        for mut receiver in receiver.iter_mut() {
            for message in receiver.receive() {
                info!("Received ping: {:?}", message);

                let response = PingMessage(format!("Pong! Got: {}", message.0));
                if let Err(e) = sender.send::<_, Channel1>(&response, server, &NetworkTarget::All) {
                    error!("Failed to send response: {:?}", e);
                }
            }
        }
    }
}
