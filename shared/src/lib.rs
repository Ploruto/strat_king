use bevy::prelude::*;
use core::net::{IpAddr, Ipv4Addr, SocketAddr};
use core::time::Duration;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

use crate::gameplay::map::{GamePath, GameStructure, StructureConnections};
use crate::gameplay::structures::{Tower, TowerStats};

pub const FIXED_TIMESTEP_HZ: f64 = 64.0;
pub const SERVER_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 7777);
pub const SERVER_REPLICATION_INTERVAL: Duration = Duration::from_millis(100);

pub mod gameplay;

#[derive(Clone)]
pub struct SharedPlugin;

pub struct Channel1;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PingMessage(pub String);

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<PingMessage>()
            .add_direction(NetworkDirection::Bidirectional);

        app.add_channel::<Channel1>(ChannelSettings {
            mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
            ..default()
        })
        .add_direction(NetworkDirection::Bidirectional);

        // Register replicated components
        app.register_component::<GameStructure>();

        app.register_component::<GamePath>();
        //     .add_direction(NetworkDirection::ServerToClient);

        app.register_component::<StructureConnections>();
        // app.add_replication::<StructureConnections>()
        //     .add_direction(NetworkDirection::ServerToClient);

        app.register_component::<Tower>();
        // app.add_replication::<Tower>()
        //     .add_direction(NetworkDirection::ServerToClient);

        app.register_component::<TowerStats>();
        // app.add_replication::<TowerStats>()
        //     .add_direction(NetworkDirection::ServerToClient);
    }
}
