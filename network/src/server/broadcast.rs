//! Broadcast channel for WebSocket state push.

use tokio::sync::broadcast;

/// Create a broadcast channel for WebSocket state updates.
#[must_use]
pub fn create_broadcast(capacity: usize) -> broadcast::Sender<Vec<u8>> {
    broadcast::channel(capacity).0
}
