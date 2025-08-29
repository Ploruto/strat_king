use crate::gameplay::map::TeamId;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize, Clone, PartialEq)]
pub struct TowerStats {
    level: u8,                // 1
    max_mana: u8,             // 30
    regen_rate: f32,          // 2
    overflow_degen_rate: f32, // 6
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

#[derive(Component)]
pub struct BaseTowerMarker;

#[derive(Component, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tower {
    pub mana: u8,
    pub owner: Option<TeamId>,
}
