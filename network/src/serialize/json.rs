//! JSON serialization helpers for protocol messages.

use crate::NetworkError;

/// Encode a value to JSON bytes.
///
/// # Errors
/// Returns [`NetworkError::Serialization`] if serialization fails.
pub fn encode_json<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, NetworkError> {
    serde_json::to_vec(value).map_err(|e| NetworkError::Serialization(e.to_string()))
}

/// Decode a value from JSON bytes.
///
/// # Errors
/// Returns [`NetworkError::Serialization`] if deserialization fails.
pub fn decode_json<'a, T: serde::de::Deserialize<'a>>(buf: &'a [u8]) -> Result<T, NetworkError> {
    serde_json::from_slice(buf).map_err(|e| NetworkError::Serialization(e.to_string()))
}
