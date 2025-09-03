use bevy::prelude::*;
use bevy::prelude::*;
use godot::classes::*;
use godot_bevy::prelude::*;

#[derive(Default)]
pub struct ExampleButtonPlugin {}

impl Plugin for ExampleButtonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, connect_signals);
        app.add_systems(Update, handle_signals);
    }
}

fn connect_signals(mut scene_tree: SceneTreeRef, signals: GodotSignals) {
    if let Some(root) = scene_tree.get().get_root() {
        if let Some(button) = root.try_get_node_as::<Button>("Node2D/ExampleButton") {
            let mut handle = GodotNodeHandle::from_instance_id(button.instance_id());
            signals.connect(&mut handle, "pressed");
        }
        if let Some(button) = root.try_get_node_as::<Button>("Node2D/ExampleButton2") {
            let mut handle = GodotNodeHandle::from_instance_id(button.instance_id());
            signals.connect(&mut handle, "pressed");
        }
    }
}

fn handle_signals(mut signal_events: EventReader<GodotSignal>, mut scene_tree: SceneTreeRef) {
    for signal in signal_events.read() {
        match signal.name.as_str() {
            "pressed" => {
                println!("Button was pressed!");
            }
            "toggled" => {
                println!("Toggle button changed state");
            }
            _ => {}
        }
    }
}
