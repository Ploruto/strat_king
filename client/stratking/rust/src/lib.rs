use bevy::prelude::*;
use bevy_tokio_tasks::TokioTasksPlugin;
use godot::prelude::*;
use godot_bevy::prelude::*;

use crate::example_button_binding::ExampleButtonPlugin;
use crate::networking::{LoginRequest, LoginRequested, NetworkingPlugin};

pub mod example_button_binding;
pub mod networking;

#[bevy_app]
fn build_app(app: &mut App) {
    // GodotDefaultPlugins provides all standard godot-bevy functionality
    app.add_plugins(GodotDefaultPlugins);
    app.add_plugins(TokioTasksPlugin::default());
    app.add_plugins(ExampleButtonPlugin::default());
    app.add_plugins(NetworkingPlugin);
}
