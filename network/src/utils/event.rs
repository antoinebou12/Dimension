//! Event envelope: scope, event name, payload for pub-sub style messaging.

use serde::{Deserialize, Serialize};

/// Generic event envelope for scope + event + payload.
///
/// No full pub-sub runtime—just a serializable envelope so apps can send
/// "scope + event + payload" over existing transports and dispatch on the other side.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    /// Namespace for the event (e.g. session id, channel).
    pub scope: String,
    /// Event name (e.g. "move", "chat").
    pub event: String,
    /// Opaque payload (e.g. JSON or binary).
    pub payload: Vec<u8>,
    /// Optional sequence number for ordering (use with [SequenceGenerator](crate::utils::SequenceGenerator)).
    pub sequence: Option<u64>,
    /// Optional timestamp in milliseconds.
    pub timestamp_ms: Option<u64>,
}

impl EventEnvelope {
    /// Create an envelope with scope, event, and payload.
    #[must_use]
    pub fn new(scope: impl Into<String>, event: impl Into<String>, payload: Vec<u8>) -> Self {
        Self {
            scope: scope.into(),
            event: event.into(),
            payload,
            sequence: None,
            timestamp_ms: None,
        }
    }

    /// Set optional sequence number.
    #[must_use]
    pub fn with_sequence(mut self, sequence: u64) -> Self {
        self.sequence = Some(sequence);
        self
    }

    /// Set optional timestamp (milliseconds).
    #[must_use]
    pub fn with_timestamp_ms(mut self, timestamp_ms: u64) -> Self {
        self.timestamp_ms = Some(timestamp_ms);
        self
    }
}
