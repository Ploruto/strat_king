use bevy::prelude::*;
use bevy_tokio_tasks::TokioTasksPlugin;
use godot::prelude::*;
use godot_bevy::prelude::*;

use crate::{
    example_button_binding::TestingNetworkPlugin,
    networking::{LoginRequest, LoginRequested, MatchFound, NetworkingPlugin},
};

pub mod example_button_binding;
pub mod networking;

#[bevy_app]
fn build_app(app: &mut App) {
    // GodotDefaultPlugins provides all standard godot-bevy functionality
    app.add_plugins(GodotDefaultPlugins);
    app.add_plugins(TokioTasksPlugin::default());
    app.add_plugins(TestingNetworkPlugin::default());
    app.add_plugins(NetworkingPlugin);

    app.add_systems(Update, handle_match_found);
}

fn handle_match_found(mut events: EventReader<MatchFound>) {
    for game in events.read() {
        info!("Handle Match: {:?}", game)
    }
}
