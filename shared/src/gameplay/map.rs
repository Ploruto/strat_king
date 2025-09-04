use crate::gameplay::structures::StructureType;
use bevy::prelude::*;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

pub type NodeId = u16;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MapNode {
    pub id: NodeId,
    pub connected_to: Vec<NodeId>,
    pub position: Vec2,
    pub node_type: NodeType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Map {
    pub name: String,
    pub nodes: HashMap<NodeId, MapNode>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NodeType {
    StructureType(StructureType),
    Waypoint,
}

pub struct MapNodeData {
    pub id: NodeId,
    pub connected_to: &'static [NodeId],
    pub position: Vec2,
    pub node_type: NodeType,
}

pub struct MapData {
    pub name: &'static str,
    pub nodes: &'static [MapNodeData],
}

impl Map {
    pub fn from_const(map_data: &MapData) -> Self {
        let nodes = map_data
            .nodes
            .iter()
            .map(|node_data| {
                let node = MapNode {
                    id: node_data.id,
                    connected_to: node_data.connected_to.to_vec(),
                    position: node_data.position,
                    node_type: node_data.node_type.clone(),
                };
                (node_data.id, node)
            })
            .collect();

        let map = Self {
            name: map_data.name.to_string(),
            nodes,
        };

        map.validate_undirected()
            .expect("Map must be an undirected graph");
        map
    }

    pub fn get_node(&self, id: NodeId) -> Option<&MapNode> {
        self.nodes.get(&id)
    }

    pub fn get_connected_nodes(&self, id: NodeId) -> Option<Vec<&MapNode>> {
        self.get_node(id).map(|node| {
            node.connected_to
                .iter()
                .filter_map(|&connected_id| self.get_node(connected_id))
                .collect()
        })
    }

    fn validate_undirected(&self) -> Result<(), String> {
        for (node_id, node) in &self.nodes {
            for &connected_id in &node.connected_to {
                if let Some(connected_node) = self.nodes.get(&connected_id) {
                    if !connected_node.connected_to.contains(node_id) {
                        return Err(format!(
                            "Graph is not undirected: node {} connects to {} but {} doesn't connect back",
                            node_id, connected_id, connected_id
                        ));
                    }
                } else {
                    return Err(format!(
                        "Node {} references non-existent node {}",
                        node_id, connected_id
                    ));
                }
            }
        }
        Ok(())
    }
}

pub const EXAMPLE_MAP: MapData = MapData {
    name: "Example Map",
    nodes: &[
        MapNodeData {
            id: 1,
            connected_to: &[2, 3],
            position: Vec2::new(0.0, 0.0),
            node_type: NodeType::StructureType(StructureType::Tower(None)),
        },
        MapNodeData {
            id: 2,
            connected_to: &[1, 3, 4],
            position: Vec2::new(100.0, 0.0),
            node_type: NodeType::Waypoint,
        },
        MapNodeData {
            id: 3,
            connected_to: &[1, 2, 4],
            position: Vec2::new(50.0, 86.6),
            node_type: NodeType::StructureType(StructureType::BaseTower(1)),
        },
        MapNodeData {
            id: 4,
            connected_to: &[2, 3],
            position: Vec2::new(150.0, 86.6),
            node_type: NodeType::StructureType(StructureType::Tower(Some(2))),
        },
    ],
};

#[derive(Component, Serialize, Deserialize, PartialEq)]
pub struct CurrentMap(pub Map);

fn setup_current_map(mut commands: Commands) {
    let default_map = Map::from_const(&EXAMPLE_MAP);
    commands.spawn(CurrentMap(default_map));
}

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_current_map)
            .register_component::<CurrentMap>();
    }
}
