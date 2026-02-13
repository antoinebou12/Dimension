//! Axis-aligned BSP tree for spatial queries.
//!
//! Build from `(T, Aabb)` pairs; supports frustum and ray intersection.

use crate::aabb::ray_aabb;
use crate::{Aabb, Frustum};

const LEAF_THRESHOLD: usize = 4;

/// BSP tree node: either a leaf with items or an inner node with a split plane.
#[derive(Debug)]
pub enum BspNode<T> {
    /// Leaf: items and their AABBs.
    Leaf {
        /// Bounding box of all items in this leaf.
        bounds: Aabb,
        /// Items and their AABBs.
        items: Vec<(T, Aabb)>,
    },
    /// Inner: split along one axis.
    Inner {
        /// Split axis: 0 = X, 1 = Y, 2 = Z.
        axis: u8,
        /// Split position along that axis.
        split: f32,
        /// Bounding box of this subtree.
        bounds: Aabb,
        /// Child nodes.
        left: Box<BspNode<T>>,
        right: Box<BspNode<T>>,
    },
}

/// Axis-aligned BSP tree for frustum and ray queries.
#[derive(Debug)]
pub struct BspTree<T> {
    /// Root node; empty if built from no items.
    root: Option<Box<BspNode<T>>>,
}

impl<T: Copy + Clone> BspTree<T> {
    /// Builds a BSP tree from item–AABB pairs.
    ///
    /// Splits alternate X, Y, Z by depth. Leaves hold up to `LEAF_THRESHOLD` items.
    #[must_use]
    pub fn build(items: &[(T, Aabb)]) -> Self {
        if items.is_empty() {
            return Self { root: None };
        }
        let bounds = union_aabbs(items.iter().map(|(_, a)| a));
        let root = Some(Box::new(Self::build_node(items, 0, bounds)));
        Self { root }
    }

    fn build_node(items: &[(T, Aabb)], depth: usize, bounds: Aabb) -> BspNode<T> {
        if items.len() <= LEAF_THRESHOLD {
            return BspNode::Leaf {
                bounds,
                items: items.to_vec(),
            };
        }
        let axis = (depth % 3) as u8;
        let mut items_sorted: Vec<(T, Aabb)> = items.to_vec();
        items_sorted.sort_by(|(_, a), (_, b)| {
            let ca = (a.min[axis as usize] + a.max[axis as usize]) * 0.5;
            let cb = (b.min[axis as usize] + b.max[axis as usize]) * 0.5;
            ca.partial_cmp(&cb).unwrap_or(std::cmp::Ordering::Equal)
        });
        let mid = items_sorted.len() / 2;
        let split = {
            let (_, a) = &items_sorted[mid];
            (a.min[axis as usize] + a.max[axis as usize]) * 0.5
        };
        let (left_slice, right_slice) = items_sorted.split_at(mid);
        let left_bounds = union_aabbs(left_slice.iter().map(|(_, a)| a));
        let right_bounds = union_aabbs(right_slice.iter().map(|(_, a)| a));
        let left = Box::new(Self::build_node(left_slice, depth + 1, left_bounds));
        let right = Box::new(Self::build_node(right_slice, depth + 1, right_bounds));
        BspNode::Inner {
            axis,
            split,
            bounds,
            left,
            right,
        }
    }

    /// Returns items whose AABB intersects the frustum.
    #[must_use]
    pub fn intersect_frustum(&self, frustum: &Frustum) -> Vec<T> {
        let mut out = Vec::new();
        if let Some(ref root) = self.root {
            Self::intersect_frustum_node(root, frustum, &mut out);
        }
        out
    }

    fn intersect_frustum_node(node: &BspNode<T>, frustum: &Frustum, out: &mut Vec<T>) {
        if !frustum.intersects_aabb(node.bounds()) {
            return;
        }
        match node {
            BspNode::Leaf { items, .. } => {
                for (id, aabb) in items {
                    if frustum.intersects_aabb(aabb) {
                        out.push(*id);
                    }
                }
            }
            BspNode::Inner { left, right, .. } => {
                Self::intersect_frustum_node(left, frustum, out);
                Self::intersect_frustum_node(right, frustum, out);
            }
        }
    }

    /// Returns items whose AABB is hit by the ray (origin + t*dir, t >= 0).
    #[must_use]
    pub fn intersect_ray(&self, origin: &[f32; 3], dir: &[f32; 3]) -> Vec<T> {
        let mut out = Vec::new();
        if let Some(ref root) = self.root {
            Self::intersect_ray_node(root, origin, dir, &mut out);
        }
        out
    }

    fn intersect_ray_node(node: &BspNode<T>, origin: &[f32; 3], dir: &[f32; 3], out: &mut Vec<T>) {
        if ray_aabb(origin, dir, node.bounds()).is_none() {
            return;
        }
        match node {
            BspNode::Leaf { items, .. } => {
                for (id, aabb) in items {
                    if ray_aabb(origin, dir, aabb).is_some() {
                        out.push(*id);
                    }
                }
            }
            BspNode::Inner { left, right, .. } => {
                Self::intersect_ray_node(left, origin, dir, out);
                Self::intersect_ray_node(right, origin, dir, out);
            }
        }
    }
}

impl<T> BspNode<T> {
    fn bounds(&self) -> &Aabb {
        match self {
            BspNode::Leaf { bounds, .. } => bounds,
            BspNode::Inner { bounds, .. } => bounds,
        }
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

#[cfg(test)]
mod tests {
    use super::{BspNode, BspTree};
    use crate::{Aabb, Frustum};
    use mathlib::cg::matrix4f_identity;

    #[test]
    fn bsp_build_empty() {
        let tree: BspTree<u32> = BspTree::build(&[]);
        assert!(tree.root.is_none());
    }

    #[test]
    fn bsp_build_single() {
        let items = [(0u32, Aabb::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]))];
        let tree = BspTree::build(&items);
        assert!(tree.root.is_some());
        let r = tree.root.as_ref().unwrap().as_ref();
        match r {
            BspNode::Leaf {
                items: leaf_items, ..
            } => {
                assert_eq!(leaf_items.len(), 1);
                assert_eq!(leaf_items[0].0, 0);
            }
            _ => panic!("expected leaf"),
        }
    }

    #[test]
    fn bsp_intersect_ray() {
        let items = [
            (0u32, Aabb::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0])),
            (1u32, Aabb::new([2.0, 0.0, 0.0], [3.0, 1.0, 1.0])),
        ];
        let tree = BspTree::build(&items);
        let origin = [0.5, 0.5, 2.0];
        let dir = [0.0, 0.0, -1.0];
        let hit = tree.intersect_ray(&origin, &dir);
        assert_eq!(hit.len(), 1);
        assert_eq!(hit[0], 0);
    }

    #[test]
    fn bsp_intersect_frustum() {
        let items = [
            (0u32, Aabb::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0])),
            (
                1u32,
                Aabb::new([100.0, 100.0, 100.0], [101.0, 101.0, 101.0]),
            ),
        ];
        let tree = BspTree::build(&items);
        let view_proj = matrix4f_identity();
        let frustum = Frustum::from_view_proj(&view_proj);
        let visible = tree.intersect_frustum(&frustum);
        assert!(!visible.is_empty());
    }

    #[test]
    fn bsp_intersect_ray_empty() {
        let items = [
            (0u32, Aabb::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0])),
            (1u32, Aabb::new([2.0, 0.0, 0.0], [3.0, 1.0, 1.0])),
        ];
        let tree = BspTree::build(&items);
        let origin = [5.0, 5.0, 5.0];
        let dir = [1.0, 0.0, 0.0];
        let hit = tree.intersect_ray(&origin, &dir);
        assert!(hit.is_empty());
    }

    #[test]
    fn bsp_intersect_frustum_empty() {
        let items = [
            (0u32, Aabb::new([10.0, 10.0, 10.0], [11.0, 11.0, 11.0])),
            (1u32, Aabb::new([20.0, 20.0, 20.0], [21.0, 21.0, 21.0])),
        ];
        let tree = BspTree::build(&items);
        let view_proj = matrix4f_identity();
        let frustum = Frustum::from_view_proj(&view_proj);
        let visible = tree.intersect_frustum(&frustum);
        assert!(visible.is_empty());
    }
}
