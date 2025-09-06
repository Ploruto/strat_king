use bevy::prelude::*;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

use crate::GameNetworkChannel;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameState {
    Running,
    Paused,
}

#[derive(Component, Serialize, Deserialize, Clone, PartialEq)]
pub struct CurrentGameState(pub GameState);

pub fn run_if_game_running(game_state: Query<&CurrentGameState>) -> bool {
    let Ok(state) = game_state.single() else {
        error!("Run_if_game_running called before CurrentGameState exists");
        return false;
    };
    state.0 == GameState::Running
}

fn setup_game_state(mut commands: Commands) {
    println!("Set game State!");
    commands.spawn(CurrentGameState(GameState::Running));
}

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_game_state);
        app.register_component::<CurrentGameState>();
    }
}
