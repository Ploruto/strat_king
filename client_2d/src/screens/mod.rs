//! The game's main screen states and transitions between them.

mod authentication;
mod connecting;
mod gameplay;
mod loading;
mod matchmaking;
mod splash;
mod title;

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.init_state::<Screen>();

    app.add_plugins((
        authentication::plugin,
        connecting::plugin,
        gameplay::plugin,
        loading::plugin,
        matchmaking::plugin,
        splash::plugin,
        title::plugin,
    ));
}

/// The game's main screen states.
#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
#[states(scoped_entities)]
pub enum Screen {
    #[default]
    Splash,
    Title,
    Loading,
    Authentication,
    Matchmaking,
    Connecting,
    Gameplay,
}
