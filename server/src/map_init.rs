use bevy::prelude::*;
use lightyear::prelude::{NetworkTarget, Replicate};
use shared::gameplay::map::{
    GamePath, GameStructure, MapDefinition, PathId, StructureConnections, StructureId,
    StructureType, TeamId,
};
use shared::gameplay::structures::{BaseTowerMarker, Tower, TowerStats};
use std::collections::HashMap;

/// Resource to store the current map asset handle
#[derive(Resource)]
pub struct CurrentMap(pub Handle<MapDefinition>);

/// Resource to map definition IDs to runtime entities
#[derive(Resource)]
pub struct EntityMapping {
    pub structures: HashMap<StructureId, Entity>,
    pub paths: HashMap<PathId, Entity>,
}

/// System to initialize the map from a MapDefinition asset
pub fn initialize_map_system(
    mut commands: Commands,
    current_map: Res<CurrentMap>,
    map_assets: Res<Assets<MapDefinition>>,
    mut entity_mapping: ResMut<EntityMapping>,
) {
    // Only run if we have a map to load and haven't loaded it yet
    if entity_mapping.structures.is_empty() {
        if let Some(map_def) = map_assets.get(&current_map.0) {
            spawn_map_entities(&mut commands, map_def, &mut entity_mapping);
            info!(
                "Map '{}' initialized with {} structures and {} paths",
                map_def.name,
                map_def.structures.len(),
                map_def.paths.len()
            );
        }
    }
}

/// Spawns all entities from the map definition
fn spawn_map_entities(
    commands: &mut Commands,
    map_def: &MapDefinition,
    entity_mapping: &mut EntityMapping,
) {
    // First pass: spawn all structures
    for structure_def in &map_def.structures {
        let entity = spawn_structure(commands, structure_def);
        entity_mapping.structures.insert(structure_def.id, entity);
    }

    // Second pass: spawn all paths (now that we have structure entities)
    for path_def in &map_def.paths {
        let structure_a = *entity_mapping
            .structures
            .get(&path_def.structure_a)
            .expect("Structure A not found in entity mapping");
        let structure_b = *entity_mapping
            .structures
            .get(&path_def.structure_b)
            .expect("Structure B not found in entity mapping");

        let path_entity = spawn_path(commands, path_def, structure_a, structure_b);
        entity_mapping.paths.insert(path_def.id, path_entity);
    }

    // Third pass: link structures to their connected paths
    for structure_def in &map_def.structures {
        let structure_entity = *entity_mapping.structures.get(&structure_def.id).unwrap();
        let connected_paths = find_connected_paths(map_def, structure_def.id, entity_mapping);

        commands
            .entity(structure_entity)
            .insert(StructureConnections { connected_paths });
    }
}

/// Spawns a single structure entity
fn spawn_structure(
    commands: &mut Commands,
    structure_def: &shared::gameplay::map::StructureDefinition,
) -> Entity {
    let mut entity_commands = commands.spawn((
        GameStructure {
            definition_id: structure_def.id,
            position: structure_def.pos,
        },
        Transform::from_translation(structure_def.pos.extend(0.0)),
        Replicate::to_clients(NetworkTarget::All),
    ));

    match &structure_def.structure_type {
        StructureType::Base { team } => {
            entity_commands.insert((
                BaseTowerMarker,
                Tower {
                    mana: 30, // Start with full mana
                    owner: Some(*team),
                },
                TowerStats::new(1),
            ));
        }
        StructureType::Tower { team } => {
            entity_commands.insert((
                Tower {
                    mana: if team.is_some() { 30 } else { 0 }, // Team towers start with mana, neutral empty
                    owner: *team,
                },
                TowerStats::new(1),
            ));
        }
    }

    entity_commands.id()
}

/// Spawns a single path entity
fn spawn_path(
    commands: &mut Commands,
    path_def: &shared::gameplay::map::PathDefinition,
    structure_a: Entity,
    structure_b: Entity,
) -> Entity {
    commands
        .spawn((
            GamePath {
                definition_id: path_def.id,
                structure_a,
                structure_b,
                waypoints: path_def.waypoints.clone(),
                width: path_def.width,
            },
            Replicate::to_clients(NetworkTarget::All),
        ))
        .id()
}

/// Finds all path entities connected to a structure
fn find_connected_paths(
    map_def: &MapDefinition,
    structure_id: StructureId,
    entity_mapping: &EntityMapping,
) -> Vec<Entity> {
    map_def
        .paths_from_structure(structure_id)
        .iter()
        .map(|path_def| {
            *entity_mapping
                .paths
                .get(&path_def.id)
                .expect("Path entity not found in mapping")
        })
        .collect()
}

/// Plugin to add map initialization systems
pub struct MapInitPlugin;

impl Plugin for MapInitPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(EntityMapping {
            structures: HashMap::new(),
            paths: HashMap::new(),
        })
        .add_systems(Update, initialize_map_system);
    }
}
