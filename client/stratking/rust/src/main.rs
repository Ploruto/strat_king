use bevy::prelude::*;

mod client_logic;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins);

    // Use shared client logic
    client_logic::setup_client_app(&mut app);

    app.run();
}
