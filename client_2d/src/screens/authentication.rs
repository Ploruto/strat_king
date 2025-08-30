//! The authentication screen for online play.

use bevy::prelude::*;
use crate::{networking::NetworkingState, screens::Screen, theme::widget};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Authentication), spawn_auth_ui);
    app.add_systems(Update, handle_authentication.run_if(in_state(Screen::Authentication)));
}

#[derive(Component)]
struct AuthenticationStatus {
    is_authenticating: bool,
}

fn spawn_auth_ui(mut commands: Commands) {
    commands.spawn((
        widget::ui_root("Authentication"),
        GlobalZIndex(1),
        StateScoped(Screen::Authentication),
        AuthenticationStatus {
            is_authenticating: false,
        },
        children![
            widget::header("Authentication"),
            widget::label("Connecting to server..."),
            widget::button("Skip (Play Offline)", skip_to_offline),
        ],
    ));
}

fn handle_authentication(
    mut auth_query: Query<&mut AuthenticationStatus>,
    mut next_screen: ResMut<NextState<Screen>>,
    time: Res<Time>,
) {
    // Simple timer-based mock authentication for now
    // In reality, this would check server connectivity and handle JWT auth
    for mut auth_status in auth_query.iter_mut() {
        if !auth_status.is_authenticating {
            auth_status.is_authenticating = true;
            info!("Starting authentication process...");
        }
        
        // Mock: Auto-authenticate after 2 seconds
        // TODO: Replace with real authentication logic
        if time.elapsed().as_secs_f32() > 2.0 {
            info!("Authentication successful!");
            next_screen.set(Screen::Matchmaking);
            break;
        }
    }
}

fn skip_to_offline(
    _: Trigger<Pointer<Click>>,
    mut next_screen: ResMut<NextState<Screen>>,
) {
    info!("Skipping to offline mode");
    next_screen.set(Screen::Gameplay);
}