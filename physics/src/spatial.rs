//! Broad phase for collision: AABB overlap and candidate pair generation.
//!
//! Uses the collision crate's `Aabb` and `BvhTree`. Overlapping pairs are found
//! so that narrow-phase (contact constraints) can be built.

use collision::{Aabb, BvhTree};
use std::ops::Range;

/// Returns true if two AABBs overlap (convenience around [collision::Aabb]).
#[inline]
#[must_use]
pub fn aabb_overlap(a: &Aabb, b: &Aabb) -> bool {
    a.intersects(b)
}

/// Builds a BVH from (id, AABB) pairs for spatial queries (e.g. ray, frustum).
///
/// Overlapping pairs are obtained via [broad_phase_pairs].
#[must_use]
pub fn build_bvh(items: &[(usize, Aabb)]) -> BvhTree<usize> {
    BvhTree::build(items)
}

/// Finds all pairs (i, j) with i < j whose AABBs overlap.
///
/// Skips pairs where `skip_range` contains i or j (e.g. kinematic indices).
#[must_use]
pub fn broad_phase_pairs(
    items: &[(usize, Aabb)],
    skip_range: Option<Range<usize>>,
) -> Vec<(usize, usize)> {
    let mut pairs = Vec::new();
    let skip = |id: usize| -> bool { skip_range.as_ref().map_or(false, |r| r.contains(&id)) };
    for (idx_a, (id_a, aabb_a)) in items.iter().enumerate() {
        if skip(*id_a) {
            continue;
        }
        for (id_b, aabb_b) in items[idx_a + 1..].iter() {
            if skip(*id_b) {
                continue;
            }
            if aabb_a.intersects(aabb_b) {
                pairs.push((*id_a, *id_b));
            }
        }
    }
    pairs
}
