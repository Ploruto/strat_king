//! The authentication screen for online play.

use crate::{
    networking::{self, NetworkingState},
    screens::Screen,
    theme::widget,
};
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Authentication), spawn_auth_ui);
    app.add_systems(
        Update,
        handle_authentication.run_if(in_state(Screen::Authentication)),
    );
}

#[derive(Component)]
struct AuthenticationUI;

#[derive(Component)]
struct UsernameInput;

#[derive(Component)]
struct PasswordInput;

#[derive(Component)]
struct LoginButton;

#[derive(Component)]
struct RegisterButton;

#[derive(Component)]
struct StatusMessage;

#[derive(Component)]
struct AuthenticationStatus {
    is_authenticating: bool,
    username: String,
    password: String,
    status_message: String,
}

fn spawn_auth_ui(mut commands: Commands) {
    commands.spawn((
        widget::ui_root("Authentication"),
        GlobalZIndex(1),
        StateScoped(Screen::Authentication),
        AuthenticationUI,
        AuthenticationStatus {
            is_authenticating: false,
            username: String::new(),
            password: String::new(),
            status_message: "Enter your credentials to play online".to_string(),
        },
        children![
            widget::header("Strategy King - Login"),
            // Status message
            (
                widget::label("Choose player to login as:"),
                StatusMessage
            ),
            // Player 1 login button
            (widget::button("Login as Player 1", handle_player1_login), LoginButton),
            // Player 2 login button
            (
                widget::button("Login as Player 2", handle_player2_login),
                RegisterButton
            ),
            // Skip to offline button
            widget::button("Skip (Play Offline)", skip_to_offline),
        ],
    ));
}

fn handle_authentication(
    auth_query: Query<&AuthenticationStatus>,
    mut networking_state: ResMut<NetworkingState>,
    mut next_screen: ResMut<NextState<Screen>>,
) {
    // This system now handles the result of authentication attempts
    // The actual authentication is triggered by button clicks
    for auth_status in auth_query.iter() {
        if auth_status.is_authenticating {
            // Authentication is in progress, UI will show loading state
            // info!("Authentication in progress...");
        }
    }
}

// Placeholder button handlers (simplified for now)
fn select_username(_: Trigger<Pointer<Click>>) {
    info!("Username field selected (TODO: implement text input)");
}

fn select_password(_: Trigger<Pointer<Click>>) {
    info!("Password field selected (TODO: implement text input)");
}

fn handle_player1_login(
    _: Trigger<Pointer<Click>>,
    mut auth_query: Query<&mut AuthenticationStatus>,
    networking_state: Res<NetworkingState>,
    runtime: Res<bevy_tokio_tasks::TokioTasksRuntime>,
) {
    authenticate_player("testuser", "password123", auth_query, networking_state, runtime);
}

fn handle_player2_login(
    _: Trigger<Pointer<Click>>,
    mut auth_query: Query<&mut AuthenticationStatus>,
    networking_state: Res<NetworkingState>,
    runtime: Res<bevy_tokio_tasks::TokioTasksRuntime>,
) {
    authenticate_player("player2", "password123", auth_query, networking_state, runtime);
}

fn authenticate_player(
    username: &str,
    password: &str,
    mut auth_query: Query<&mut AuthenticationStatus>,
    networking_state: Res<NetworkingState>,
    runtime: Res<bevy_tokio_tasks::TokioTasksRuntime>,
) {
    info!("Login button clicked for {}", username);

    for mut auth_status in auth_query.iter_mut() {
        if !auth_status.is_authenticating {
            auth_status.is_authenticating = true;
            auth_status.status_message = format!("Authenticating as {}...", username);

            let username = username.to_string();
            let password = password.to_string();
            let server_url = networking_state.server_url.clone();

            // Spawn async task for authentication using bevy-tokio-tasks
            runtime.spawn_background_task(|mut ctx| async move {
                match networking::authenticate_user(&username, &password, &server_url).await {
                    Ok(response) => {
                        info!(
                            "Authentication successful! Player ID: {}, Username: {}",
                            response.data.player_id, response.data.username
                        );

                        // Update networking state and navigate to matchmaking
                        let auth_token = response.data.token.clone();
                        let player_id = response.data.player_id.to_string();

                        ctx.run_on_main_thread(move |ctx| {
                            // Update networking state
                            if let Some(mut networking_state) =
                                ctx.world.get_resource_mut::<NetworkingState>()
                            {
                                networking_state.is_connected = true;
                                networking_state.auth_token = Some(auth_token);
                                networking_state.player_id = Some(player_id);
                            }

                            // Navigate to matchmaking screen
                            if let Some(mut next_screen) =
                                ctx.world.get_resource_mut::<NextState<Screen>>()
                            {
                                next_screen.set(Screen::Matchmaking);
                            }
                        })
                        .await;
                    }
                    Err(error) => {
                        error!("Authentication failed: {}", error);

                        // Update UI with error message on main thread
                        ctx.run_on_main_thread(move |ctx| {
                            // TODO: Update authentication status with error message
                        })
                        .await;
                    }
                }
            });
        }
    }
}

fn skip_to_offline(_: Trigger<Pointer<Click>>, mut next_screen: ResMut<NextState<Screen>>) {
    info!("Skipping to offline mode");
    next_screen.set(Screen::Gameplay);
}
