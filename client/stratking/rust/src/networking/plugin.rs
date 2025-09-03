use bevy::prelude::*;
use crate::networking::{NetworkManager, websocket::websocket_system, http::*};
use crate::networking::events::*;

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        app
            // Add the NetworkManager resource
            .init_resource::<NetworkManager>()
            
            // Register all networking events
            .add_event::<LoginRequested>()
            .add_event::<LogoutRequested>()
            .add_event::<JoinQueueRequested>()
            .add_event::<LeaveQueueRequested>()
            .add_event::<StartOfflineGameRequested>()
            .add_event::<SyncNowRequested>()
            
            // Response events
            .add_event::<LoginCompleted>()
            .add_event::<LogoutCompleted>()
            .add_event::<MatchFound>()
            .add_event::<QueueJoined>()
            .add_event::<QueueLeft>()
            .add_event::<ConnectionEstablished>()
            .add_event::<ConnectionLost>()
            .add_event::<SyncCompleted>()
            .add_event::<NetworkError>()
            
            // Add networking systems
            .add_systems(Update, (
                websocket_system,
                http_system,
                queue_system,
            ));
    }
}