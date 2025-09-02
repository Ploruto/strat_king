// Support configuring Bevy lints within code.
#![cfg_attr(bevy_lint, feature(register_tool), register_tool(bevy))]
// Disable console on Windows for non-dev builds.
#![cfg_attr(not(feature = "dev"), windows_subsystem = "windows")]

mod asset_tracking;
mod audio;
mod demo;
#[cfg(feature = "dev")]
mod dev_tools;
mod menus;
mod networking;
mod screens;
mod theme;

use bevy::{asset::AssetMetaCheck, prelude::*};
use lightyear::prelude::{MessageReceiver, MessageSender};
use shared::{Channel1, PingMessage};

use crate::networking::NetworkingState;

fn main() -> AppExit {
    App::new().add_plugins(AppPlugin).run()
}

fn send_ping(
    mut timer: Local<Timer>,
    time: Res<Time>,
    mut sender: Query<&mut MessageSender<PingMessage>>,
    networking_state: Res<NetworkingState>,
) {
    if timer.duration() == core::time::Duration::ZERO {
        *timer = Timer::from_seconds(5.0, TimerMode::Repeating);
    }

    timer.tick(time.delta());

    if timer.just_finished() {
        for mut sender in sender.iter_mut() {
            if let Some(player_id) = &networking_state.player_id {
                let msg = format!("Ping from client with player_id: {}", player_id);
                let ping = PingMessage(msg.to_string());
                sender.send::<Channel1>(ping);
                info!("Sent ping message");
            }
        }
    }
}

fn handle_pong(mut receiver: Query<&mut MessageReceiver<PingMessage>>) {
    for mut receiver in receiver.iter_mut() {
        for message in receiver.receive() {
            info!("Pong from server: {}", message.0);
        }
    }
}

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (handle_pong, send_ping));
        // Add Bevy plugins.
        app.add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    // Wasm builds will check for meta files (that don't exist) if this isn't set.
                    // This causes errors and even panics on web build on itch.
                    // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Window {
                        title: "Client 2d".to_string(),
                        fit_canvas_to_parent: true,
                        ..default()
                    }
                    .into(),
                    ..default()
                }),
        );

        // Add other plugins.
        app.add_plugins((
            asset_tracking::plugin,
            audio::plugin,
            demo::plugin,
            #[cfg(feature = "dev")]
            dev_tools::plugin,
            menus::plugin,
            networking::plugin,
            screens::plugin,
            theme::plugin,
            bevy_tokio_tasks::TokioTasksPlugin::default(),
            // Add Lightyear client plugins
            lightyear::prelude::client::ClientPlugins {
                tick_duration: core::time::Duration::from_secs_f64(1.0 / shared::FIXED_TIMESTEP_HZ),
            },
            // Add shared protocol
            shared::SharedPlugin,
        ));

        // Order new `AppSystems` variants by adding them here:
        app.configure_sets(
            Update,
            (
                AppSystems::TickTimers,
                AppSystems::RecordInput,
                AppSystems::Update,
            )
                .chain(),
        );

        // Set up the `Pause` state.
        app.init_state::<Pause>();
        app.configure_sets(Update, PausableSystems.run_if(in_state(Pause(false))));

        // Spawn the main camera.
        app.add_systems(Startup, spawn_camera);
    }
}

/// High-level groupings of systems for the app in the `Update` schedule.
/// When adding a new variant, make sure to order it in the `configure_sets`
/// call above.
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
enum AppSystems {
    /// Tick timers.
    TickTimers,
    /// Record player input.
    RecordInput,
    /// Do everything else (consider splitting this into further variants).
    Update,
}

/// Whether or not the game is paused.
#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
#[states(scoped_entities)]
struct Pause(pub bool);

/// A system set for systems that shouldn't run while the game is paused.
#[derive(SystemSet, Copy, Clone, Eq, PartialEq, Hash, Debug)]
struct PausableSystems;

fn spawn_camera(mut commands: Commands) {
    commands.spawn((Name::new("Camera"), Camera2d));
}
