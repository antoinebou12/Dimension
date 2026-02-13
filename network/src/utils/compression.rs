//! Delta compression utilities for state updates.

use crate::serialize::binary;
use crate::utils::delta;
use crate::NetworkError;

/// Compute a minimal delta between two world snapshot buffers (protobuf-encoded).
///
/// Decodes `from` and `to` as [`WorldSnapshot`], computes the delta, and returns
/// the encoded [`crate::protocol::proto::WorldDelta`].
///
/// # Errors
/// Returns [`NetworkError::Serialization`] if either buffer cannot be decoded as WorldSnapshot.
pub fn compute_delta_bytes(from: &[u8], to: &[u8]) -> Result<Vec<u8>, NetworkError> {
    let from_snap = binary::decode_world_snapshot(from)?;
    let to_snap = binary::decode_world_snapshot(to)?;
    let delta = delta::compute_delta_snapshots(&from_snap, &to_snap);
    binary::encode_proto(&delta)
}
