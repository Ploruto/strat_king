use bevy::prelude::*;
use lightyear::prelude::*;
use shared::gameplay::{
    map::{CurrentMap, NodeType},
    structures::{BaseTowerMarker, Tower, TowerStats},
};

use crate::GameState;

/// Plugin to add map initialization systems
pub struct MapInitPlugin;

impl Plugin for MapInitPlugin {
    fn build(&self, app: &mut App) {
        // app.add_systems(OnEnter(GameState::GameRunning), spawn_map);
    }
}

pub fn spawn_map(mut commands: Commands, q_map: Query<&CurrentMap>) {
    let map = q_map.single().unwrap().clone();
    // Spawn structures based on node type
    for (node_id, node) in &map.0.nodes {
        match &node.node_type {
            NodeType::StructureType(structure_type) => {
                match structure_type {
                    shared::gameplay::structures::StructureType::Tower(owner) => {
                        commands.spawn((
                            Tower {
                                mana: 0,
                                node_id: *node_id,
                                owner: *owner,
                            },
                            TowerStats::new(1),
                            Transform::from_translation(node.position.extend(0.0)),
                            GlobalTransform::default(),
                        ));
                    }
                    shared::gameplay::structures::StructureType::BaseTower(team_id) => {
                        commands.spawn((
                            Tower {
                                mana: 30, // Base towers start with full mana
                                node_id: *node_id,
                                owner: Some(*team_id),
                            },
                            TowerStats::new(1),
                            BaseTowerMarker,
                            Transform::from_translation(node.position.extend(0.0)),
                            GlobalTransform::default(),
                        ));
                    }
                }
            }
            NodeType::Waypoint => {
                // Waypoints don't spawn entities
            }
        }
    }
}

pub fn despawn_all_towers(mut commands: Commands, towers: Query<Entity, With<Tower>>) {
    for entity in towers.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
