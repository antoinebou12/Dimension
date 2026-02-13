//! Protobuf binary encode/decode helpers.

use prost::Message;

use crate::protocol::proto;
use crate::NetworkError;

/// Encode a protobuf message to bytes.
///
/// # Errors
/// Returns [`NetworkError::Serialization`] if encoding fails.
pub fn encode_proto<M: Message>(msg: &M) -> Result<Vec<u8>, NetworkError> {
    let mut buf = Vec::with_capacity(msg.encoded_len());
    msg.encode(&mut buf)
        .map_err(|e| NetworkError::Serialization(e.to_string()))?;
    Ok(buf)
}

/// Decode a protobuf message from bytes.
///
/// # Errors
/// Returns [`NetworkError::Serialization`] if decoding fails.
pub fn decode_proto<M: Message + Default>(buf: &[u8]) -> Result<M, NetworkError> {
    M::decode(buf).map_err(|e| NetworkError::Serialization(e.to_string()))
}

/// Encode a Vec3 to protobuf bytes.
///
/// # Errors
/// Returns [`NetworkError::Serialization`] if encoding fails.
pub fn encode_vec3(v: &proto::Vec3) -> Result<Vec<u8>, NetworkError> {
    encode_proto(v)
}

/// Decode a Vec3 from protobuf bytes.
///
/// # Errors
/// Returns [`NetworkError::Serialization`] if decoding fails.
pub fn decode_vec3(buf: &[u8]) -> Result<proto::Vec3, NetworkError> {
    decode_proto(buf)
}

/// Encode a WorldSnapshot to protobuf bytes.
///
/// # Errors
/// Returns [`NetworkError::Serialization`] if encoding fails.
pub fn encode_world_snapshot(msg: &proto::WorldSnapshot) -> Result<Vec<u8>, NetworkError> {
    encode_proto(msg)
}

/// Decode a WorldSnapshot from protobuf bytes.
///
/// # Errors
/// Returns [`NetworkError::Serialization`] if decoding fails.
pub fn decode_world_snapshot(buf: &[u8]) -> Result<proto::WorldSnapshot, NetworkError> {
    decode_proto(buf)
}
