use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use lightyear::prelude::server::*;
use lightyear::prelude::*;
use shared::*;
use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use crate::map_init::{CurrentMap, MapInitPlugin};
use shared::gameplay::map::MapDefinition;

mod map_init;

#[derive(Resource)]
pub struct ServerConfig {
    pub auth_token_secret: String,
    pub match_id: String,
    pub expected_players: usize,
    pub server_port: u16,
    pub server_addr: SocketAddr,
}

fn main() {
    let server_port = env::var("SERVER_PORT")
        .unwrap_or_else(|_| "5000".to_string())
        .parse::<u16>()
        .expect("Invalid SERVER_PORT");

    let server_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), server_port);

    let server_config = ServerConfig {
        auth_token_secret: env::var("AUTH_TOKEN_SECRET").unwrap_or_else(|_| "default_secret".to_string()),
        match_id: env::var("MATCH_ID").unwrap_or_else(|_| "default_match".to_string()),
        expected_players: env::var("EXPECTED_PLAYERS")
            .unwrap_or_else(|_| "2".to_string())
            .parse()
            .expect("Invalid EXPECTED_PLAYERS"),
        server_port,
        server_addr,
    };

    println!("Starting game server for match: {}", server_config.match_id);
    println!("Server listening on: {}", server_config.server_addr);
    println!("Expected players: {}", server_config.expected_players);

    let mut app = App::new();

    app.insert_resource(server_config);
    
    // app.add_plugins(MinimalPlugins);
    app.add_plugins(DefaultPlugins);
    app.add_plugins(ServerPlugins {
        tick_duration: core::time::Duration::from_secs_f64(1.0 / FIXED_TIMESTEP_HZ),
    });

    app.add_plugins(SharedPlugin);
    app.add_plugins(ServerPlugin);

    app.run();
}

pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<MapDefinition>();
        app.add_plugins(RonAssetPlugin::<MapDefinition>::new(&[".ron"]));
        app.add_systems(Startup, (startup, load_map_asset));
        app.add_observer(handle_new_client);
        app.add_systems(Update, handle_ping_message);
        app.add_plugins(MapInitPlugin);
    }
}

fn handle_new_client(trigger: Trigger<OnAdd, Connected>, mut commands: Commands) {
    info!("New client connected: {:?}", trigger.target());
    commands.entity(trigger.target()).insert((
        ReplicationSender::new(
            SERVER_REPLICATION_INTERVAL,
            SendUpdatesMode::SinceLastAck,
            false,
        ),
        // MessageReceiver::<PingMessage>::default(),
    ));
}

fn startup(mut commands: Commands, config: Res<ServerConfig>) {
    info!("Starting server on {:?}", config.server_addr);
    let server = commands
        .spawn((
            NetcodeServer::new(NetcodeConfig::default()),
            LocalAddr(config.server_addr),
            ServerUdpIo::default(),
        ))
        .id();
    commands.trigger_targets(Start, server);
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
