use bevy::prelude::*;
use godot::prelude::*;
use godot_bevy::prelude::*;

use crate::example_button_binding::ExampleButtonPlugin;

pub mod example_button_binding;

#[bevy_app]
fn build_app(app: &mut App) {
    // GodotDefaultPlugins provides all standard godot-bevy functionality
    app.add_plugins(GodotDefaultPlugins);
    app.add_plugins(ExampleButtonPlugin::default());

    // Add your systems here
    app.add_systems(Update, hello_world_system);
}

fn hello_world_system(mut timer: Local<f32>, time: Res<Time>) {
    // This runs every frame in Bevy's Update schedule
    *timer += time.delta_secs();
    if *timer > 1.0 {
        *timer = 0.0;
        godot_print!("Hello from Bevy ECS!");
    }
}

fn handle_button_press(mut events: EventReader<GodotSignal>) {
    for signal in events.read() {
        if signal.name == "pressed" {
            // Button was pressed!
        }
    }
}
