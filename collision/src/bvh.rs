//! Bounding Volume Hierarchy (BVH) for spatial queries.
//!
//! Bounding Volume Hierarchy (BVH) for spatial queries.
//!
//! Build from `(T, Aabb)` pairs; supports frustum and ray intersection.
//! Supports median split (default), binned SAH, or Morton (LBVH) build.

use crate::aabb::ray_aabb;
use crate::{Aabb, Frustum};

const LEAF_THRESHOLD: usize = 4;
const NUM_SAH_BINS: usize = 8;
/// Bits per axis for Morton code (10 bits → 60-bit codes, fits in u64).
const MORTON_BITS: u32 = 10;
const MORTON_MAX: u32 = (1 << MORTON_BITS) - 1;

/// Build strategy for the BVH.
///
/// - **Median**: Fast; split at median along the longest axis. Good default.
/// - **Sah**: Binned Surface Area Heuristic; better tree quality, slower build.
/// - **Morton**: LBVH; sort by Morton (Z-order) code once, split at median. Fast build, good spatial coherence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BvhBuildStrategy {
    /// Median split along the longest axis (default).
    #[default]
    Median,
    /// Binned SAH: minimizes expected traversal cost (surface area × count).
    Sah,
    /// Morton (LBVH): sort by Z-order code, split at median. Single global sort, cache-friendly.
    Morton,
}

/// Spreads lower 10 bits of `v` into 64-bit with zeros between each bit (for 3D Morton interleave).
#[inline]
fn spread_bits(v: u32) -> u64 {
    let mut x = (v & MORTON_MAX) as u64;
    x = (x | (x << 32)) & 0x0000_0001_0000_0001;
    x = (x | (x << 16)) & 0x0001_0001_0001_0001;
    x = (x | (x << 8)) & 0x0101_0101_0101_0101;
    x = (x | (x << 4)) & 0x1111_1111_1111_1111;
    x
}

/// 3D Morton (Z-order) code from quantized integer coordinates.
#[inline]
fn morton3d(x: u32, y: u32, z: u32) -> u64 {
    spread_bits(x) | (spread_bits(y) << 1) | (spread_bits(z) << 2)
}

/// BVH node: either a leaf with items or an inner node with two children.
#[derive(Debug)]
pub enum BvhNode<T> {
    /// Leaf: items and their AABBs.
    Leaf {
        /// Bounding box of all items in this leaf.
        bounds: Aabb,
        /// Items and their AABBs.
        items: Vec<(T, Aabb)>,
    },
    /// Inner: two children.
    Inner {
        /// Bounding box of this subtree.
        bounds: Aabb,
        /// Left child.
        left: Box<BvhNode<T>>,
        /// Right child.
        right: Box<BvhNode<T>>,
    },
}

/// BVH for frustum and ray queries.
///
/// When the `parallel` feature is enabled, `T` must be `Send + Sync` for the parallel build.
#[derive(Debug)]
pub struct BvhTree<T> {
    /// Root node; empty if built from no items.
    root: Option<Box<BvhNode<T>>>,
}

impl<T: Copy + Clone + Send + Sync> BvhTree<T> {
    /// Builds a BVH from item–AABB pairs using median split along the longest axis.
    #[must_use]
    pub fn build(items: &[(T, Aabb)]) -> Self {
        Self::build_with_strategy(items, BvhBuildStrategy::Median)
    }

    /// Builds a BVH from item–AABB pairs using the given strategy.
    ///
    /// # Examples
    ///
    /// ```
    /// # use collision::{Aabb, BvhBuildStrategy, BvhTree};
    /// let items = vec![(0u32, Aabb::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]))];
    /// let tree = BvhTree::build_with_strategy(&items, BvhBuildStrategy::Sah);
    /// ```
    #[must_use]
    pub fn build_with_strategy(items: &[(T, Aabb)], strategy: BvhBuildStrategy) -> Self {
        if items.is_empty() {
            return Self { root: None };
        }
        let bounds = union_aabbs(items.iter().map(|(_, a)| a));
        let root = Some(Box::new(match strategy {
            BvhBuildStrategy::Median => Self::build_node_median(items, bounds),
            BvhBuildStrategy::Sah => Self::build_node_sah(items, bounds),
            BvhBuildStrategy::Morton => Self::build_node_morton(items, bounds),
        }));
        Self { root }
    }

    fn build_node_median(items: &[(T, Aabb)], bounds: Aabb) -> BvhNode<T> {
        if items.len() <= LEAF_THRESHOLD {
            return BvhNode::Leaf {
                bounds,
                items: items.to_vec(),
            };
        }
        let axis = longest_axis(&bounds);
        let mut items_sorted: Vec<(T, Aabb)> = items.to_vec();
        items_sorted.sort_by(|(_, a), (_, b)| {
            let ca = (a.min[axis] + a.max[axis]) * 0.5;
            let cb = (b.min[axis] + b.max[axis]) * 0.5;
            ca.partial_cmp(&cb).unwrap_or(std::cmp::Ordering::Equal)
        });
        let mid = items_sorted.len() / 2;
        let (left_slice, right_slice) = items_sorted.split_at(mid);
        let left_bounds = union_aabbs(left_slice.iter().map(|(_, a)| a));
        let right_bounds = union_aabbs(right_slice.iter().map(|(_, a)| a));

        #[cfg(all(feature = "parallel", not(target_arch = "wasm32")))]
        let (left, right) = {
            use par_core::join;
            let (left_node, right_node) = join(
                || Self::build_node_median(left_slice, left_bounds),
                || Self::build_node_median(right_slice, right_bounds),
            );
            (Box::new(left_node), Box::new(right_node))
        };

        #[cfg(not(all(feature = "parallel", not(target_arch = "wasm32"))))]
        let (left, right) = (
            Box::new(Self::build_node_median(left_slice, left_bounds)),
            Box::new(Self::build_node_median(right_slice, right_bounds)),
        );

        BvhNode::Inner {
            bounds,
            left,
            right,
        }
    }

    fn build_node_sah(items: &[(T, Aabb)], bounds: Aabb) -> BvhNode<T> {
        if items.len() <= LEAF_THRESHOLD {
            return BvhNode::Leaf {
                bounds,
                items: items.to_vec(),
            };
        }
        let axis = longest_axis(&bounds);
        let centroid_min = items
            .iter()
            .map(|(_, a)| (a.min[axis] + a.max[axis]) * 0.5)
            .fold(f32::INFINITY, f32::min);
        let centroid_max = items
            .iter()
            .map(|(_, a)| (a.min[axis] + a.max[axis]) * 0.5)
            .fold(f32::NEG_INFINITY, f32::max);
        let range = centroid_max - centroid_min;
        if range <= 0.0 {
            return Self::build_node_median(items, bounds);
        }
        let inv_range = 1.0 / range;
        let mut bins: Vec<(Aabb, usize)> = (0..NUM_SAH_BINS)
            .map(|_| (Aabb::new([f32::INFINITY; 3], [f32::NEG_INFINITY; 3]), 0))
            .collect();
        for (_, aabb) in items {
            let c = (aabb.min[axis] + aabb.max[axis]) * 0.5;
            let bin_id = ((c - centroid_min) * inv_range * (NUM_SAH_BINS as f32))
                .floor()
                .clamp(0.0, (NUM_SAH_BINS - 1) as f32) as usize;
            let (ref_aabb, count) = &mut bins[bin_id];
            *ref_aabb = ref_aabb.union(aabb);
            *count += 1;
        }
        let mut best_cost = f32::INFINITY;
        let mut best_split = NUM_SAH_BINS / 2;
        let mut left_merge = Aabb::new([f32::INFINITY; 3], [f32::NEG_INFINITY; 3]);
        let mut left_count: usize = 0;
        for i in 0..NUM_SAH_BINS.saturating_sub(1) {
            let (b, c) = &bins[i];
            left_merge = left_merge.union(b);
            left_count += *c;
            let mut right_merge = Aabb::new([f32::INFINITY; 3], [f32::NEG_INFINITY; 3]);
            let mut right_count: usize = 0;
            for (b, c) in bins[(i + 1)..].iter() {
                right_merge = right_merge.union(b);
                right_count += *c;
            }
            if left_count == 0 || right_count == 0 {
                continue;
            }
            let cost = aabb_surface_area(&left_merge) * (left_count as f32)
                + aabb_surface_area(&right_merge) * (right_count as f32);
            if cost < best_cost {
                best_cost = cost;
                best_split = i + 1;
            }
        }
        let split_val = centroid_min + (best_split as f32 / NUM_SAH_BINS as f32) * range;
        let mut items_sorted: Vec<(T, Aabb)> = items.to_vec();
        items_sorted.sort_by(|(_, a), (_, b)| {
            let ca = (a.min[axis] + a.max[axis]) * 0.5;
            let cb = (b.min[axis] + b.max[axis]) * 0.5;
            ca.partial_cmp(&cb).unwrap_or(std::cmp::Ordering::Equal)
        });
        let mid = items_sorted
            .iter()
            .position(|(_, a)| (a.min[axis] + a.max[axis]) * 0.5 >= split_val)
            .unwrap_or(items_sorted.len() / 2);
        let mid = mid.max(1).min(items_sorted.len().saturating_sub(1));
        let (left_slice, right_slice) = items_sorted.split_at(mid);
        let left_bounds = union_aabbs(left_slice.iter().map(|(_, a)| a));
        let right_bounds = union_aabbs(right_slice.iter().map(|(_, a)| a));

        #[cfg(all(feature = "parallel", not(target_arch = "wasm32")))]
        let (left, right) = {
            use par_core::join;
            let (left_node, right_node) = join(
                || Self::build_node_sah(left_slice, left_bounds),
                || Self::build_node_sah(right_slice, right_bounds),
            );
            (Box::new(left_node), Box::new(right_node))
        };

        #[cfg(not(all(feature = "parallel", not(target_arch = "wasm32"))))]
        let (left, right) = (
            Box::new(Self::build_node_sah(left_slice, left_bounds)),
            Box::new(Self::build_node_sah(right_slice, right_bounds)),
        );

        BvhNode::Inner {
            bounds,
            left,
            right,
        }
    }

    fn build_node_morton(items: &[(T, Aabb)], bounds: Aabb) -> BvhNode<T> {
        if items.len() <= LEAF_THRESHOLD {
            return BvhNode::Leaf {
                bounds,
                items: items.to_vec(),
            };
        }
        let centroid_min: [f32; 3] = [
            items
                .iter()
                .map(|(_, a)| (a.min[0] + a.max[0]) * 0.5)
                .fold(f32::INFINITY, f32::min),
            items
                .iter()
                .map(|(_, a)| (a.min[1] + a.max[1]) * 0.5)
                .fold(f32::INFINITY, f32::min),
            items
                .iter()
                .map(|(_, a)| (a.min[2] + a.max[2]) * 0.5)
                .fold(f32::INFINITY, f32::min),
        ];
        let centroid_max: [f32; 3] = [
            items
                .iter()
                .map(|(_, a)| (a.min[0] + a.max[0]) * 0.5)
                .fold(f32::NEG_INFINITY, f32::max),
            items
                .iter()
                .map(|(_, a)| (a.min[1] + a.max[1]) * 0.5)
                .fold(f32::NEG_INFINITY, f32::max),
            items
                .iter()
                .map(|(_, a)| (a.min[2] + a.max[2]) * 0.5)
                .fold(f32::NEG_INFINITY, f32::max),
        ];
        let range: [f32; 3] = [
            centroid_max[0] - centroid_min[0],
            centroid_max[1] - centroid_min[1],
            centroid_max[2] - centroid_min[2],
        ];
        if range[0] <= 0.0 && range[1] <= 0.0 && range[2] <= 0.0 {
            return Self::build_node_median(items, bounds);
        }
        let inv_range: [f32; 3] = [
            if range[0] > 0.0 { 1.0 / range[0] } else { 0.0 },
            if range[1] > 0.0 { 1.0 / range[1] } else { 0.0 },
            if range[2] > 0.0 { 1.0 / range[2] } else { 0.0 },
        ];
        let mut items_sorted: Vec<(T, Aabb)> = items.to_vec();
        items_sorted.sort_by_key(|(_, a)| {
            let cx = (a.min[0] + a.max[0]) * 0.5;
            let cy = (a.min[1] + a.max[1]) * 0.5;
            let cz = (a.min[2] + a.max[2]) * 0.5;
            let ux = ((cx - centroid_min[0]) * inv_range[0]).clamp(0.0, 1.0);
            let uy = ((cy - centroid_min[1]) * inv_range[1]).clamp(0.0, 1.0);
            let uz = ((cz - centroid_min[2]) * inv_range[2]).clamp(0.0, 1.0);
            let ix = (ux * (MORTON_MAX as f32)).round() as u32;
            let it = (uy * (MORTON_MAX as f32)).round() as u32;
            let iz = (uz * (MORTON_MAX as f32)).round() as u32;
            morton3d(ix, it, iz)
        });
        let mid = items_sorted.len() / 2;
        let (left_slice, right_slice) = items_sorted.split_at(mid);
        let left_bounds = union_aabbs(left_slice.iter().map(|(_, a)| a));
        let right_bounds = union_aabbs(right_slice.iter().map(|(_, a)| a));

        #[cfg(all(feature = "parallel", not(target_arch = "wasm32")))]
        let (left, right) = {
            use par_core::join;
            let (left_node, right_node) = join(
                || Self::build_node_morton(left_slice, left_bounds),
                || Self::build_node_morton(right_slice, right_bounds),
            );
            (Box::new(left_node), Box::new(right_node))
        };

        #[cfg(not(all(feature = "parallel", not(target_arch = "wasm32"))))]
        let (left, right) = (
            Box::new(Self::build_node_morton(left_slice, left_bounds)),
            Box::new(Self::build_node_morton(right_slice, right_bounds)),
        );

        BvhNode::Inner {
            bounds,
            left,
            right,
        }
    }

    /// Returns items whose AABB intersects the frustum.
    #[must_use]
    pub fn intersect_frustum(&self, frustum: &Frustum) -> Vec<T> {
        self.intersect_frustum_iter(frustum).collect()
    }

    /// Returns an iterator over items whose AABB intersects the frustum (depth-first order).
    ///
    /// # Examples
    ///
    /// ```
    /// # use collision::{Aabb, BvhTree, Frustum};
    /// # use mathlib::cg::matrix4f_identity;
    /// let items = vec![(0u32, Aabb::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]))];
    /// let tree = BvhTree::build(&items);
    /// let view_proj = matrix4f_identity();
    /// let frustum = Frustum::from_view_proj(&view_proj);
    /// let visible: Vec<u32> = tree.intersect_frustum_iter(&frustum).collect();
    /// ```
    #[must_use]
    pub fn intersect_frustum_iter<'a>(&'a self, frustum: &'a Frustum) -> BvhFrustumIter<'a, T> {
        BvhFrustumIter::new(self, frustum)
    }

    /// Returns items whose AABB is hit by the ray (origin + t*dir, t >= 0).
    #[must_use]
    pub fn intersect_ray(&self, origin: &[f32; 3], dir: &[f32; 3]) -> Vec<T> {
        self.intersect_ray_iter(origin, dir).collect()
    }

    /// Returns an iterator over items whose AABB is hit by the ray (depth-first order).
    ///
    /// # Examples
    ///
    /// ```
    /// # use collision::{Aabb, BvhTree};
    /// let items = vec![(0u32, Aabb::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]))];
    /// let tree = BvhTree::build(&items);
    /// let origin = [0.5, 0.5, 2.0];
    /// let dir = [0.0, 0.0, -1.0];
    /// let first = tree.intersect_ray_iter(&origin, &dir).next();
    /// ```
    #[must_use]
    pub fn intersect_ray_iter<'a>(
        &'a self,
        origin: &[f32; 3],
        dir: &[f32; 3],
    ) -> BvhRayIter<'a, T> {
        BvhRayIter::new(self, origin, dir)
    }
}

impl<T> BvhNode<T> {
    #[allow(dead_code)] // used by internal traversal; kept for future API
    fn bounds(&self) -> &Aabb {
        match self {
            BvhNode::Leaf { bounds, .. } => bounds,
            BvhNode::Inner { bounds, .. } => bounds,
        }
    }
}

/// Iterator over items whose AABB is hit by a ray (depth-first order).
pub struct BvhRayIter<'a, T> {
    origin: [f32; 3],
    dir: [f32; 3],
    stack: Vec<&'a BvhNode<T>>,
    leaf_items: Option<std::slice::Iter<'a, (T, Aabb)>>,
}

impl<'a, T: Copy + Clone + Send + Sync> BvhRayIter<'a, T> {
    fn new(tree: &'a BvhTree<T>, origin: &[f32; 3], dir: &[f32; 3]) -> Self {
        let mut stack = Vec::new();
        if let Some(ref root) = tree.root {
            stack.push(root.as_ref());
        }
        Self {
            origin: *origin,
            dir: *dir,
            stack,
            leaf_items: None,
        }
    }
}

impl<'a, T: Copy + Clone + Send + Sync> Iterator for BvhRayIter<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(ref mut it) = self.leaf_items {
                for (id, aabb) in it.by_ref() {
                    if ray_aabb(&self.origin, &self.dir, aabb).is_some() {
                        return Some(*id);
                    }
                }
                self.leaf_items = None;
                continue;
            }
            let node = self.stack.pop()?;
            match node {
                BvhNode::Leaf { bounds, items } => {
                    if ray_aabb(&self.origin, &self.dir, bounds).is_none() {
                        continue;
                    }
                    self.leaf_items = Some(items.iter());
                }
                BvhNode::Inner {
                    bounds,
                    left,
                    right,
                    ..
                } => {
                    if ray_aabb(&self.origin, &self.dir, bounds).is_none() {
                        continue;
                    }
                    self.stack.push(right.as_ref());
                    self.stack.push(left.as_ref());
                }
            }
        }
    }
}

/// Iterator over items whose AABB intersects a frustum (depth-first order).
pub struct BvhFrustumIter<'a, T> {
    frustum: &'a Frustum,
    stack: Vec<&'a BvhNode<T>>,
    leaf_items: Option<std::slice::Iter<'a, (T, Aabb)>>,
}

impl<'a, T: Copy + Clone + Send + Sync> BvhFrustumIter<'a, T> {
    fn new(tree: &'a BvhTree<T>, frustum: &'a Frustum) -> Self {
        let mut stack = Vec::new();
        if let Some(ref root) = tree.root {
            stack.push(root.as_ref());
        }
        Self {
            frustum,
            stack,
            leaf_items: None,
        }
    }
}

impl<'a, T: Copy + Clone + Send + Sync> Iterator for BvhFrustumIter<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(ref mut it) = self.leaf_items {
                for (id, aabb) in it.by_ref() {
                    if self.frustum.intersects_aabb(aabb) {
                        return Some(*id);
                    }
                }
                self.leaf_items = None;
                continue;
            }
            let node = self.stack.pop()?;
            match node {
                BvhNode::Leaf { bounds, items } => {
                    if !self.frustum.intersects_aabb(bounds) {
                        continue;
                    }
                    self.leaf_items = Some(items.iter());
                }
                BvhNode::Inner {
                    bounds,
                    left,
                    right,
                    ..
                } => {
                    if !self.frustum.intersects_aabb(bounds) {
                        continue;
                    }
                    self.stack.push(right.as_ref());
                    self.stack.push(left.as_ref());
                }
            }
        }
    }
}

fn longest_axis(bounds: &Aabb) -> usize {
    let d = [
        bounds.max[0] - bounds.min[0],
        bounds.max[1] - bounds.min[1],
        bounds.max[2] - bounds.min[2],
    ];
    if d[0] >= d[1] && d[0] >= d[2] {
        0
    } else if d[1] >= d[2] {
        1
    } else {
        2
    }
}

fn union_aabbs<'a>(aabbs: impl Iterator<Item = &'a Aabb>) -> Aabb {
    let mut min = [f32::INFINITY; 3];
    let mut max = [f32::NEG_INFINITY; 3];
    for a in aabbs {
        for i in 0..3 {
            min[i] = min[i].min(a.min[i]);
            max[i] = max[i].max(a.max[i]);
        }
    }
    Aabb::new(min, max)
}

fn aabb_surface_area(a: &Aabb) -> f32 {
    let h = a.half_extents();
    if h[0] <= 0.0 || h[1] <= 0.0 || h[2] <= 0.0 {
        return 0.0;
    }
    2.0 * (h[0] * h[1] + h[0] * h[2] + h[1] * h[2])
}

#[cfg(test)]
mod tests {
    use super::{BvhNode, BvhTree};
    use crate::{Aabb, Frustum};
    use mathlib::cg::matrix4f_identity;

    #[test]
    fn bvh_build_empty() {
        let tree: BvhTree<u32> = BvhTree::build(&[]);
        assert!(tree.root.is_none());
    }

    #[test]
    fn bvh_build_single() {
        let items = [(0u32, Aabb::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]))];
        let tree = BvhTree::build(&items);
        assert!(tree.root.is_some());
        let r = tree.root.as_ref().unwrap().as_ref();
        match r {
            BvhNode::Leaf {
                items: leaf_items, ..
            } => {
                assert_eq!(leaf_items.len(), 1);
                assert_eq!(leaf_items[0].0, 0);
            }
            _ => panic!("expected leaf"),
        }
    }

    #[test]
    fn bvh_intersect_ray() {
        let items = [
            (0u32, Aabb::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0])),
            (1u32, Aabb::new([2.0, 0.0, 0.0], [3.0, 1.0, 1.0])),
        ];
        let tree = BvhTree::build(&items);
        let origin = [0.5, 0.5, 2.0];
        let dir = [0.0, 0.0, -1.0];
        let hit = tree.intersect_ray(&origin, &dir);
        assert_eq!(hit.len(), 1);
        assert_eq!(hit[0], 0);
    }

    #[test]
    fn bvh_intersect_frustum() {
        let items = [
            (0u32, Aabb::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0])),
            (
                1u32,
                Aabb::new([100.0, 100.0, 100.0], [101.0, 101.0, 101.0]),
            ),
        ];
        let tree = BvhTree::build(&items);
        let view_proj = matrix4f_identity();
        let frustum = Frustum::from_view_proj(&view_proj);
        let visible = tree.intersect_frustum(&frustum);
        assert!(!visible.is_empty());
    }

    #[test]
    fn bvh_intersect_ray_empty() {
        let items = [
            (0u32, Aabb::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0])),
            (1u32, Aabb::new([2.0, 0.0, 0.0], [3.0, 1.0, 1.0])),
        ];
        let tree = BvhTree::build(&items);
        let origin = [5.0, 5.0, 5.0];
        let dir = [1.0, 0.0, 0.0];
        let hit = tree.intersect_ray(&origin, &dir);
        assert!(hit.is_empty());
    }

    #[test]
    fn bvh_intersect_frustum_empty() {
        let items = [
            (0u32, Aabb::new([10.0, 10.0, 10.0], [11.0, 11.0, 11.0])),
            (1u32, Aabb::new([20.0, 20.0, 20.0], [21.0, 21.0, 21.0])),
        ];
        let tree = BvhTree::build(&items);
        let view_proj = matrix4f_identity();
        let frustum = Frustum::from_view_proj(&view_proj);
        let visible = tree.intersect_frustum(&frustum);
        assert!(visible.is_empty());
    }
}
