//! Serialization for physics state (feature `serde`).

#![cfg(feature = "serde")]

use crate::state::PhysicsState;

/// Serializes the physics state to JSON.
///
/// # Errors
/// Returns an error if serialization fails.
pub fn to_json(state: &PhysicsState) -> Result<String, serde_json::Error> {
    serde_json::to_string(state)
}

/// Deserializes the physics state from JSON.
///
/// # Errors
/// Returns an error if the string is not valid JSON or does not match the state format.
pub fn from_json(s: &str) -> Result<PhysicsState, serde_json::Error> {
    serde_json::from_str(s)
}

/// Serializes the physics state to compact binary bytes (bincode).
///
/// # Errors
/// Returns an error if serialization fails.
pub fn to_bytes(state: &PhysicsState) -> Result<Vec<u8>, bincode::Error> {
    bincode::serialize(state)
}

/// Deserializes the physics state from binary bytes.
///
/// # Errors
/// Returns an error if the bytes are not valid.
pub fn from_bytes(bytes: &[u8]) -> Result<PhysicsState, bincode::Error> {
    bincode::deserialize(bytes)
}
