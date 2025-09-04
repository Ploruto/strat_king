use crate::gameplay::{
    map::{Map, NodeId, NodeType},
    state::{CurrentGameState, GameState, run_if_game_running},
    *,
};
use bevy::prelude::*;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum StructureType {
    Tower(Option<TeamId>),
    BaseTower(TeamId),
}

#[derive(Component, Serialize, Deserialize, PartialEq)]
pub struct BaseTowerMarker;

#[derive(Component, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tower {
    pub mana: u8,
    pub node_id: NodeId,
    pub owner: Option<TeamId>,
}

pub type TeamId = u8;

#[derive(Component, Serialize, Deserialize, Clone, PartialEq)]
pub struct TowerStats {
    level: u8,                // 1
    max_mana: u8,             // 30
    regen_rate: f32,          // 2
    overflow_degen_rate: f32, // 6
}

#[derive(Component, Serialize, Deserialize, PartialEq)]
pub struct TowerGenerationTimer {
    pub elapsed: f32,
    pub duration: f32,
    pub finished: bool,
}

impl TowerGenerationTimer {
    pub fn new(duration: f32) -> Self {
        Self {
            elapsed: 0.0,
            duration,
            finished: false,
        }
    }

    pub fn tick(&mut self, delta: f32) {
        self.elapsed += delta;
        if self.elapsed >= self.duration {
            self.finished = true;
        }
    }

    pub fn reset(&mut self) {
        self.elapsed = 0.0;
        self.finished = false;
    }

    pub fn finished(&self) -> bool {
        self.finished
    }
}

fn generate_mana_for_captured_towers(
    time: Res<Time>,
    mut gen_timer: Query<&mut TowerGenerationTimer>,
    mut q_towers: Query<(&mut Tower, &TowerStats)>,
) {
    let Ok(mut timer) = gen_timer.get_single_mut() else {
        return;
    };

    timer.tick(time.delta_secs());

    if timer.finished() {
        for (mut tower, stats) in q_towers.iter_mut() {
            if tower.owner.is_some() {
                if tower.mana < stats.max_mana() {
                    tower.mana =
                        (tower.mana as f32 + stats.regen_rate()).min(stats.max_mana() as f32) as u8;
                } else if tower.mana > stats.max_mana() {
                    tower.mana = (tower.mana as f32 - stats.overflow_degen_rate())
                        .max(stats.max_mana() as f32) as u8;
                }
            }
            info!(
                "Tower at node {} (owner: {:?}) has {} mana",
                tower.node_id, tower.owner, tower.mana
            );
        }
        timer.reset();
    }
}

pub struct TowerPlugin;

fn setup_tower_timer(mut commands: Commands) {
    commands.spawn(TowerGenerationTimer::new(1.0));
}

impl Plugin for TowerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_tower_timer).add_systems(
            Update,
            generate_mana_for_captured_towers.run_if(run_if_game_running),
        );
        app.register_component::<Tower>();
        app.register_component::<TowerStats>();
        app.register_component::<BaseTowerMarker>();
        app.register_component::<TowerGenerationTimer>();
    }
}

impl TowerStats {
    pub fn level(&self) -> u8 {
        self.level
    }

    pub fn max_mana(&self) -> u8 {
        self.max_mana
    }

    pub fn regen_rate(&self) -> f32 {
        self.regen_rate
    }

    pub fn overflow_degen_rate(&self) -> f32 {
        self.overflow_degen_rate
    }

    pub fn new(level: u8) -> Self {
        match level {
            1 => TowerStats {
                level: 1,
                max_mana: 30,
                regen_rate: 2.0,
                overflow_degen_rate: 6.0,
            },
            _ => panic!("Unsupported tower level: {}", level),
        }
    }
}
