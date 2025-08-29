use bevy::prelude::*;
use lightyear::prelude::server::*;
use lightyear::prelude::*;
use shared::*;

use crate::map_init::{MapInitPlugin, CurrentMap};
use shared::gameplay::map::MapDefinition;

mod map_init;

fn main() {
    let mut app = App::new();

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

fn startup(mut commands: Commands) {
    info!("Starting server on {:?}", SERVER_ADDR);
    let server = commands
        .spawn((
            NetcodeServer::new(NetcodeConfig::default()),
            LocalAddr(SERVER_ADDR),
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
