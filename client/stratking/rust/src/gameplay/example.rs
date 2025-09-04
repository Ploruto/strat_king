use bevy::prelude::*;
use godot::prelude::*;
use godot_bevy::prelude::*;

#[derive(Default)]
pub struct GameplayExamplePlugin {}

impl Plugin for GameplayExamplePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, check_if_player_node_exists);
    }
}

#[derive(Component, Default)]
pub struct Health(u8);

#[derive(Component, Default)]
pub struct Velocity(f32);

#[derive(Component, Default)]
pub struct PlayerMarker;

#[derive(GodotClass, BevyBundle)]
#[class(init, base=Node2D)]
#[bevy_bundle((Health), (Velocity), (PlayerMarker))]
pub struct PlayerNode {
    base: Base<Node2D>,
}

fn check_if_player_node_exists(mut q: Query<(&mut Transform), With<PlayerMarker>>) {
    for (mut p) in q.iter_mut() {
        p.translation.x += 0.5;
    }
}
