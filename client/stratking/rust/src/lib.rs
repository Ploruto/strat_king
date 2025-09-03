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

    // Add your systems here
    app.add_systems(Update, (simulate_login));
}

fn simulate_login(
    mut timer: Local<f32>,
    mut fired: Local<bool>,
    time: Res<Time>,
    mut login_writer: EventWriter<LoginRequested>,
) {
    *timer += time.delta_secs();

    if *timer > 5.0 && !*fired {
        info!("Sending login request");
        let login = LoginRequested {
            username: "testuser".into(),
            password: "password123".to_string(),
        };
        login_writer.write(login);
        *fired = true;
    }
}
