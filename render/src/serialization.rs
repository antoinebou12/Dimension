//! Compact binary serialization for scene/world. Requires the `serde` feature.
//!
//! The format preserves the full tree structure (parent/child) and all per-entity
//! data (transform, primitive, color). Use [`world_to_bytes`] and [`world_from_bytes`].

use crate::scene::World;

/// Serialize a world to a compact binary format (bincode). Preserves the full tree
/// structure (parent/child) and all per-entity data (transform, primitive, color).
///
/// # Errors
/// Returns error if serialization fails.
///
/// # Examples
/// ```
/// # #[cfg(feature = "serde")]
/// use render::{World, world_to_bytes, world_from_bytes};
/// # #[cfg(feature = "serde")]
/// # let world = World::new();
/// # #[cfg(feature = "serde")]
/// # let bytes = world_to_bytes(&world).unwrap();
/// # #[cfg(feature = "serde")]
/// # let _restored = world_from_bytes(&bytes).unwrap();
/// ```
#[cfg(feature = "serde")]
pub fn world_to_bytes(world: &World) -> Result<Vec<u8>, bincode::Error> {
    bincode::serialize(world)
}

/// Deserialize a world from bytes produced by [`world_to_bytes`].
///
/// # Errors
/// Returns error if the bytes are not valid or the format is incompatible.
#[cfg(feature = "serde")]
pub fn world_from_bytes(bytes: &[u8]) -> Result<World, bincode::Error> {
    bincode::deserialize(bytes)
}
