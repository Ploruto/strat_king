use bevy::prelude::*;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type StructureId = u8;
pub type PathId = u8;
pub type TeamId = u8;

#[derive(Asset, TypePath, Serialize, Deserialize, Clone)]
pub struct MapDefinition {
    pub name: String,
    pub structures: Vec<StructureDefinition>,
    pub paths: Vec<PathDefinition>,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum StructureType {
    Base { team: TeamId },
    Tower { team: Option<TeamId> },
}

#[derive(Serialize, Deserialize, Clone)]
pub struct StructureDefinition {
    pub id: StructureId,
    pub pos: Vec2,
    pub structure_type: StructureType,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PathDefinition {
    pub id: PathId,
    pub structure_a: StructureId,
    pub structure_b: StructureId,
    pub waypoints: Vec<Vec2>,
    pub width: f32,
}

// Runtime ECS components (shared between client and server)
#[derive(Component, Serialize, Deserialize, Clone, PartialEq)]
pub struct GamePath {
    pub definition_id: PathId,
    pub structure_a: Entity,
    pub structure_b: Entity,
    pub waypoints: Vec<Vec2>,
    pub width: f32,
}

#[derive(Component, Serialize, Deserialize, Clone, PartialEq)]
pub struct GameStructure {
    pub definition_id: StructureId,
    pub position: Vec2,
}

#[derive(Component, Serialize, Deserialize, Clone, PartialEq)]
pub struct StructureConnections {
    pub connected_paths: Vec<Entity>,
}

impl MapDefinition {
    /// Find all paths connected to a specific structure
    pub fn paths_from_structure(&self, structure_id: StructureId) -> Vec<&PathDefinition> {
        self.paths
            .iter()
            .filter(|path| path.structure_a == structure_id || path.structure_b == structure_id)
            .collect()
    }

    /// Get waypoints for traveling from one structure to another via a specific path
    /// Returns waypoints in the correct order (from_structure -> to_structure)
    pub fn get_waypoints_to(
        &self,
        path_id: PathId,
        from_structure: StructureId,
        to_structure: StructureId,
    ) -> Option<Vec<Vec2>> {
        let path = self.paths.iter().find(|p| p.id == path_id)?;

        if path.structure_a == from_structure && path.structure_b == to_structure {
            // Forward direction - use waypoints as-is
            Some(path.waypoints.clone())
        } else if path.structure_b == from_structure && path.structure_a == to_structure {
            // Reverse direction - reverse the waypoints
            let mut reversed = path.waypoints.clone();
            reversed.reverse();
            Some(reversed)
        } else {
            // Invalid structure combination for this path
            None
        }
    }

    /// Find a path between two structures (returns first match)
    pub fn find_path_between(
        &self,
        from_structure: StructureId,
        to_structure: StructureId,
    ) -> Option<&PathDefinition> {
        self.paths.iter().find(|path| {
            (path.structure_a == from_structure && path.structure_b == to_structure)
                || (path.structure_b == from_structure && path.structure_a == to_structure)
        })
    }

    /// Get structure by ID
    pub fn get_structure(&self, structure_id: StructureId) -> Option<&StructureDefinition> {
        self.structures.iter().find(|s| s.id == structure_id)
    }

    /// Get path by ID
    pub fn get_path(&self, path_id: PathId) -> Option<&PathDefinition> {
        self.paths.iter().find(|p| p.id == path_id)
    }

    /// Get all neighboring structures (directly connected via paths)
    pub fn get_neighbors(&self, structure_id: StructureId) -> Vec<StructureId> {
        self.paths_from_structure(structure_id)
            .iter()
            .map(|path| {
                if path.structure_a == structure_id {
                    path.structure_b
                } else {
                    path.structure_a
                }
            })
            .collect()
    }
}
