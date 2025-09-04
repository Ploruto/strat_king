use bevy::prelude::*;
use core::net::{IpAddr, Ipv4Addr, SocketAddr};
use core::time::Duration;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

use crate::gameplay::state::{CurrentGameState, GameState};
use crate::gameplay::{
    map::MapPlugin,
    state::StatePlugin,
    structures::{Tower, TowerPlugin, TowerStats},
};

pub const FIXED_TIMESTEP_HZ: f64 = 12.0;
pub const SERVER_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 32768);
pub const SERVER_REPLICATION_INTERVAL: Duration = Duration::from_millis(100);

pub mod gameplay;

#[derive(Clone)]
pub struct SharedPlugin;

pub struct GameNetworkChannel;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PingMessage(pub String);

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        // Add gameplay plugins
        app.add_plugins((StatePlugin, MapPlugin, TowerPlugin));

        // Network setup
        app.add_message::<PingMessage>()
            .add_direction(NetworkDirection::Bidirectional);

        app.add_channel::<GameNetworkChannel>(ChannelSettings {
            mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
            ..default()
        })
        .add_direction(NetworkDirection::Bidirectional);
    }
}
