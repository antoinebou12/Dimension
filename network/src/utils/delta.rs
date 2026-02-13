//! Compute minimal WorldDelta between two world states.

use std::collections::HashMap;

use crate::protocol::proto::{EntityState, WorldDelta, WorldSnapshot};

/// Compute a minimal delta between two entity maps.
///
/// Returns changed entities (in `to` but not in `from`, or differing) and
/// removed entity ids (in `from` but not in `to`).
#[must_use]
pub fn compute_delta(
    from: &HashMap<u64, EntityState>,
    to: &HashMap<u64, EntityState>,
    from_tick: u64,
    to_tick: u64,
) -> WorldDelta {
    let changed: Vec<EntityState> = to
        .iter()
        .filter(|(id, to_entity)| {
            from.get(id)
                .map_or(true, |from_entity| from_entity != *to_entity)
        })
        .map(|(_, e)| e.clone())
        .collect();
    let removed: Vec<u64> = from
        .keys()
        .filter(|id| !to.contains_key(id))
        .copied()
        .collect();
    WorldDelta {
        changed,
        removed,
        from_tick,
        to_tick,
    }
}

/// Compute a minimal delta between two world snapshots.
#[must_use]
pub fn compute_delta_snapshots(from: &WorldSnapshot, to: &WorldSnapshot) -> WorldDelta {
    let from_map: HashMap<u64, EntityState> =
        from.entities.iter().map(|e| (e.id, e.clone())).collect();
    let to_map: HashMap<u64, EntityState> = to.entities.iter().map(|e| (e.id, e.clone())).collect();
    compute_delta(&from_map, &to_map, from.tick, to.tick)
}
