use bevy::prelude::*;
use bevy_tokio_tasks::TokioTasksPlugin;
use godot::prelude::*;
use godot_bevy::prelude::*;

mod client_logic;

// use crate::{
//     example_button_binding::TestingNetworkPlugin,
//     networking::{LoginRequest, LoginRequested, MatchFound, NetworkManager, NetworkingPlugin},
// };

// pub mod example_button_binding;
// pub mod gameplay;
// pub mod networking;

#[bevy_app]
// #[no_mangle]
pub fn android_main(app: &mut App) {
    // Android-specific plugins with disabled rendering
    app.add_plugins(
        DefaultPlugins
            .build()
            .disable::<bevy::render::RenderPlugin>()
            // .disable::<bevy::winit::WinitPlugin>()
            .disable::<bevy::pbr::PbrPlugin>()
            .disable::<bevy::sprite::SpritePlugin>()
            .disable::<bevy::ui::UiPlugin>()
            .disable::<bevy::text::TextPlugin>(),
    );

    // Use shared client logic
    client_logic::setup_client_app(app);
}

// fn handle_match_found(
//     mut events: EventReader<MatchFound>,
//     mut commands: Commands,
//     network_manager: Res<NetworkManager>,
// ) {
//     for game in events.read() {
//         info!("found match");
//         info!("current player: {:?}", &network_manager.current_player);

//         if let Some(current_player) = &network_manager.current_player {
//             info!("also got local user addr");
//             // let localhost = Ipv4Addr::new(127, 0, 0, 1);
//             let server_host_addr = game.server_host.clone();
//             let octets: Vec<u8> = server_host_addr
//                 .split('.')
//                 .map(|s| s.parse().unwrap())
//                 .collect();
//             let server_addr = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 7777); //game.server_port);
//             let auth = Authentication::Manual {
//                 server_addr: std::net::SocketAddr::V4(server_addr),
//                 client_id: current_player.user_id,
//                 private_key: Key::default(),
//                 protocol_id: 0,
//             };

//             let client = commands
//                 .spawn((
//                     Client::default(),
//                     LocalAddr(
//                         SocketAddrV4::new(Ipv4Addr::LOCALHOST, current_player.user_id as u16)
//                             .into(),
//                     ),
//                     PeerAddr(std::net::SocketAddr::V4(server_addr)),
//                     Link::new(None),
//                     ReplicationReceiver::default(),
//                     NetcodeClient::new(auth, NetcodeConfig::default()).unwrap(),
//                     UdpIo::default(),
//                 ))
//                 .id();

//             commands.trigger_targets(Connect, client);
//             info!("spawned client");
//         }
//         info!("Handle Match: {:?}", game)
//     }
// }

// fn send_ping(
//     mut timer: Local<Timer>,
//     time: Res<Time>,
//     mut sender: Query<&mut MessageSender<PingMessage>>,
// ) {
//     if timer.duration() == core::time::Duration::ZERO {
//         *timer = Timer::from_seconds(5.0, TimerMode::Repeating);
//     }

//     timer.tick(time.delta());

//     if timer.just_finished() {
//         for mut sender in sender.iter_mut() {
//             let ping = PingMessage("Hello from client!".to_string());
//             sender.send::<GameNetworkChannel>(ping);
//             info!("Sent ping message");
//         }
//     }
// }

// fn handle_pong(mut receiver: Query<&mut MessageReceiver<PingMessage>>) {
//     for mut receiver in receiver.iter_mut() {
//         for message in receiver.receive() {
//             info!("Received from server: {}", message.0);
//         }
//     }
// }
