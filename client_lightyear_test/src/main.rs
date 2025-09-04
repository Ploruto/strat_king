use bevy::prelude::*;
use core::net::{IpAddr, Ipv4Addr, SocketAddr};
use lightyear::netcode::Key;
use lightyear::prelude::client::*;
use lightyear::prelude::*;
use shared::*;
use std::net::SocketAddrV4;

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
        app.add_systems(Update, (send_ping, handle_pong));
    }
}

fn startup(mut commands: Commands) {
    info!("Starting client, connecting to {:?}", SERVER_ADDR);
    commands.spawn(Camera2d);

    let addr = Ipv4Addr::new(0, 0, 0, 0);

    let auth = Authentication::Manual {
        server_addr: SocketAddr::V4(SocketAddrV4::new(addr, 7777)),
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
            sender.send::<GameNetworkChannel>(ping);
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
