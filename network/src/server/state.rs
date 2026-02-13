//! Shared game state: per-room world snapshots.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::protocol::proto::WorldSnapshot;

/// Per-room game state.
#[derive(Debug, Default)]
pub struct RoomState {
    /// Latest world snapshot for the room.
    pub snapshot: Option<WorldSnapshot>,
}

/// Shared game state: room_id -> RoomState.
#[derive(Debug, Default)]
pub struct SharedGameState {
    rooms: HashMap<String, RoomState>,
}

impl SharedGameState {
    /// Create new shared game state.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set snapshot for a room.
    pub fn set_snapshot(&mut self, room_id: &str, snapshot: WorldSnapshot) {
        self.rooms.entry(room_id.to_string()).or_default().snapshot = Some(snapshot);
    }

    /// Get snapshot for a room.
    #[must_use]
    pub fn get_snapshot(&self, room_id: &str) -> Option<WorldSnapshot> {
        self.rooms.get(room_id).and_then(|r| r.snapshot.clone())
    }
}

/// Shared game state (Arc + RwLock).
pub type SharedGameStateHandle = Arc<RwLock<SharedGameState>>;
