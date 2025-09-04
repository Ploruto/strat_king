use crate::networking::{GameMode, JoinQueueRequested, LoginRequested};
use bevy::{
    app::{App, Plugin, Update},
    ecs::{
        event::{Event, EventReader, EventWriter},
        resource::Resource,
        system::{Res, ResMut},
    },
    log::{debug, info, warn},
};
use godot::{classes::*, obj::InstanceId};
use godot_bevy::prelude::*;

#[derive(Resource, Default)]
struct SignalConnectionState {
    connected: bool,
    login_button_id: Option<InstanceId>,
    join_queue_button_id: Option<InstanceId>,
}

#[derive(Event)]
pub struct LoginButtonPressed;

#[derive(Event)]
pub struct JoinQueueButtonPressed;

#[derive(Default)]
pub struct TestingNetworkPlugin {}

impl Plugin for TestingNetworkPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SignalConnectionState>();
        app.add_event::<LoginButtonPressed>();
        app.add_event::<JoinQueueButtonPressed>();
        app.add_systems(
            Update,
            (
                connect_signals,
                handle_signals,
                login_handler,
                join_queue_handler,
            ),
        );
    }
}

fn connect_signals(
    mut state: ResMut<SignalConnectionState>,
    mut scene_tree: SceneTreeRef,
    signals: GodotSignals,
) {
    if !state.connected {
        if let Some(root) = scene_tree.get().get_root() {
            if let Some(button) = root.try_get_node_as::<Button>("Node2D/UI/LoginButton") {
                let instance_id = button.instance_id();
                let mut handle = GodotNodeHandle::from_instance_id(instance_id);
                signals.connect(&mut handle, "pressed");
                state.login_button_id = Some(instance_id);
            }
            if let Some(button) = root.try_get_node_as::<Button>("Node2D/UI/JoinQueueButton") {
                let instance_id = button.instance_id();
                let mut handle = GodotNodeHandle::from_instance_id(instance_id);
                signals.connect(&mut handle, "pressed");
                state.join_queue_button_id = Some(instance_id);
            }
        }
        state.connected = true;
    }
}

fn handle_signals(
    state: Res<SignalConnectionState>,
    mut signal_events: EventReader<GodotSignal>,
    mut login_button_writer: EventWriter<LoginButtonPressed>,
    mut join_queue_button_writer: EventWriter<JoinQueueButtonPressed>,
) {
    for signal in signal_events.read() {
        if signal.name == "pressed" {
            let signal_instance_id = signal.origin.instance_id();

            if Some(signal_instance_id) == state.login_button_id {
                login_button_writer.write(LoginButtonPressed);
            } else if Some(signal_instance_id) == state.join_queue_button_id {
                join_queue_button_writer.write(JoinQueueButtonPressed);
            } else {
                debug!(
                    "Unhandled button press from instance ID: {}",
                    signal_instance_id
                );
            }
        }
    }
}

fn login_handler(
    mut login_button_events: EventReader<LoginButtonPressed>,
    mut scene_tree: SceneTreeRef,
    mut login_writer: EventWriter<LoginRequested>,
) {
    for _ in login_button_events.read() {
        if let Some(root) = scene_tree.get().get_root() {
            let mut username = String::new();
            let mut password = String::new();

            if let Some(username_input) =
                root.try_get_node_as::<LineEdit>("Node2D/UI/UsernameInput")
            {
                username = username_input.get_text().into();
            }

            if let Some(password_input) =
                root.try_get_node_as::<LineEdit>("Node2D/UI/PasswordInput")
            {
                password = password_input.get_text().into();
            }

            if !username.is_empty() && !password.is_empty() {
                info!("Attempting login for user: {}", username);
                let login_request = LoginRequested { username, password };
                login_writer.send(login_request);
            } else {
                warn!("Username or password is empty");
            }
        }
    }
}

fn join_queue_handler(
    mut join_queue_events: EventReader<JoinQueueButtonPressed>,
    mut scene_tree: SceneTreeRef,
    mut join_queue_writer: EventWriter<JoinQueueRequested>,
) {
    for _ in join_queue_events.read() {
        join_queue_writer.write(JoinQueueRequested {
            game_mode: GameMode::Ranked,
        });
    }
}
