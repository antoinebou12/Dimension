//! Lobby service: room creation, join, leave, matchmaking.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Room metadata.
#[derive(Debug, Clone)]
pub struct RoomInfo {
    /// Room identifier.
    pub room_id: String,
    /// Room display name.
    pub room_name: String,
    /// Current player count.
    pub current_players: u32,
    /// Maximum players allowed.
    pub max_players: u32,
}

/// In-memory lobby state.
#[derive(Debug, Default)]
pub struct LobbyState {
    rooms: HashMap<String, RoomInfo>,
}

impl LobbyState {
    /// Create a new lobby state.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a room.
    pub fn create_room(&mut self, room_name: String, max_players: u32) -> String {
        let room_id = format!("room-{}", uuid_simple());
        let info = RoomInfo {
            room_id: room_id.clone(),
            room_name,
            current_players: 0,
            max_players,
        };
        self.rooms.insert(room_id.clone(), info);
        room_id
    }

    /// Join a room. Returns true if successful.
    pub fn join_room(&mut self, room_id: &str) -> bool {
        if let Some(room) = self.rooms.get_mut(room_id) {
            if room.current_players < room.max_players {
                room.current_players += 1;
                return true;
            }
        }
        false
    }

    /// Leave a room.
    pub fn leave_room(&mut self, room_id: &str) {
        if let Some(room) = self.rooms.get_mut(room_id) {
            room.current_players = room.current_players.saturating_sub(1);
        }
    }

    /// List all rooms.
    #[must_use]
    pub fn list_rooms(&self) -> Vec<RoomInfo> {
        self.rooms.values().cloned().collect()
    }
}

fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{:x}", t)
}

/// Shared lobby state (Arc + RwLock).
pub type SharedLobbyState = Arc<RwLock<LobbyState>>;
