use bevy::prelude::*;
use core::net::{IpAddr, Ipv4Addr, SocketAddr};
use lightyear::netcode::Key;
use lightyear::prelude::client::*;
use lightyear::prelude::*;
use shared::gameplay::map::{GamePath, GameStructure, StructureConnections};
use shared::gameplay::structures::Tower;
use shared::*;

const CLIENT_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 4000);

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins);
    app.add_plugins(ClientPlugins {
        tick_duration: core::time::Duration::from_secs_f64(1.0 / FIXED_TIMESTEP_HZ),
    });

    app.add_plugins(SharedPlugin);
    app.add_plugins(ClientPlugin);

    app.run();
}

pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup);
        app.add_systems(Update, (send_ping, handle_pong, debug_replicated_entities));
    }
}

fn startup(mut commands: Commands) {
    info!("Starting client, connecting to {:?}", SERVER_ADDR);
    commands.spawn(Camera2d);

    let auth = Authentication::Manual {
        server_addr: SERVER_ADDR,
        client_id: 0,
        private_key: Key::default(),
        protocol_id: 0,
    };

    let client = commands
        .spawn((
            Client::default(),
            LocalAddr(CLIENT_ADDR),
            PeerAddr(SERVER_ADDR),
            Link::new(None),
            ReplicationReceiver::default(),
            NetcodeClient::new(auth, NetcodeConfig::default()).unwrap(),
            UdpIo::default(),
        ))
        .id();

    commands.trigger_targets(Connect, client);
}

fn send_ping(
    mut timer: Local<Timer>,
    time: Res<Time>,
    mut sender: Query<&mut MessageSender<PingMessage>>,
) {
    if timer.duration() == core::time::Duration::ZERO {
        *timer = Timer::from_seconds(2.0, TimerMode::Repeating);
    }

    timer.tick(time.delta());

    if timer.just_finished() {
        for mut sender in sender.iter_mut() {
            let ping = PingMessage("Hello from client!".to_string());
            sender.send::<Channel1>(ping);
            info!("Sent ping message");
        }
    }
}

fn handle_pong(mut receiver: Query<&mut MessageReceiver<PingMessage>>) {
    for mut receiver in receiver.iter_mut() {
        for message in receiver.receive() {
            info!("Received from server: {}", message.0);
        }
    }
}

/// Debug system to track replicated map entities
fn debug_replicated_entities(
    mut timer: Local<Timer>,
    time: Res<Time>,
    structures: Query<(Entity, &GameStructure, Option<&Tower>), Added<GameStructure>>,
    paths: Query<(Entity, &GamePath), Added<GamePath>>,
    connections: Query<(Entity, &StructureConnections), Added<StructureConnections>>,
    all_structures: Query<&GameStructure>,
    all_paths: Query<&GamePath>,
) {
    // Log newly added entities immediately
    for (entity, structure, tower) in structures.iter() {
        info!(
            "🏰 NEW STRUCTURE: Entity {:?}, ID {}, Pos {:?}",
            entity, structure.definition_id, structure.position
        );
        if let Some(tower) = tower {
            info!("   └─ Tower: Mana {}, Owner {:?}", tower.mana, tower.owner);
        }
    }

    for (entity, path) in paths.iter() {
        info!(
            "🛤️  NEW PATH: Entity {:?}, ID {}, Width {}, {} waypoints",
            entity,
            path.definition_id,
            path.width,
            path.waypoints.len()
        );
        info!(
            "   └─ Connects structures {:?} ↔ {:?}",
            path.structure_a, path.structure_b
        );
    }

    for (entity, connections) in connections.iter() {
        info!(
            "🔗 NEW CONNECTIONS: Entity {:?}, {} connected paths",
            entity,
            connections.connected_paths.len()
        );
    }

    // Periodic summary every 5 seconds
    if timer.duration() == core::time::Duration::ZERO {
        *timer = Timer::from_seconds(5.0, TimerMode::Repeating);
    }

    timer.tick(time.delta());

    if timer.just_finished() {
        let structure_count = all_structures.iter().count();
        let path_count = all_paths.iter().count();

        if structure_count > 0 || path_count > 0 {
            info!(
                "📊 REPLICATION STATUS: {} structures, {} paths replicated",
                structure_count, path_count
            );
        } else {
            info!("⏳ Waiting for map entities to replicate from server...");
        }
    }
}
