//! State sync: apply deltas, interpolation.

use std::collections::HashMap;

use crate::protocol::proto::{EntityState, WorldDelta, WorldSnapshot};

/// Entity state map: entity id -> state. Use as the in-memory world for sync.
pub type EntityStateMap = HashMap<u64, EntityState>;

/// Build an entity map from a full snapshot.
#[must_use]
pub fn snapshot_to_map(snapshot: &WorldSnapshot) -> EntityStateMap {
    snapshot
        .entities
        .iter()
        .map(|e| (e.id, e.clone()))
        .collect()
}

/// Build a world snapshot from an entity map and tick/timestamp.
#[must_use]
pub fn map_to_snapshot(map: &EntityStateMap, tick: u64, timestamp: f64) -> WorldSnapshot {
    WorldSnapshot {
        entities: map.values().cloned().collect(),
        tick,
        timestamp,
    }
}

/// Apply a world delta to the current state in place.
///
/// Merges all `changed` entities by id and removes all `removed` ids.
/// Caller may use `from_tick`/`to_tick` on the delta for ordering (e.g. ignore stale deltas).
pub fn apply_delta(state: &mut EntityStateMap, delta: &WorldDelta) {
    for entity in &delta.changed {
        state.insert(entity.id, entity.clone());
    }
    for id in &delta.removed {
        state.remove(id);
    }
}

/// Apply a world delta only if its tick range is after the current tick (optional ordering).
///
/// Returns true if the delta was applied, false if it was stale (to_tick <= current_tick).
pub fn apply_delta_if_newer(
    state: &mut EntityStateMap,
    delta: &WorldDelta,
    current_tick: u64,
) -> bool {
    if delta.to_tick <= current_tick {
        return false;
    }
    apply_delta(state, delta);
    true
}
