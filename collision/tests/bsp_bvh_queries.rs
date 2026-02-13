//! Integration tests: BSP and BVH build and query, and ray–shape APIs.

use collision::{
    ray_aabb, ray_capsule, ray_obb, ray_sphere, Aabb, BspTree, BvhBuildStrategy, BvhTree, Capsule,
    Frustum, Obb, Sphere,
};
use mathlib::cg::matrix4f_identity;
use std::collections::HashSet;

#[test]
fn bsp_ray_and_frustum() {
    let items: Vec<(u32, Aabb)> = vec![
        (0, Aabb::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0])),
        (1, Aabb::new([2.0, 0.0, 0.0], [3.0, 1.0, 1.0])),
        (2, Aabb::new([0.0, 2.0, 0.0], [1.0, 3.0, 1.0])),
    ];
    let bsp = BspTree::build(&items);
    let origin = [0.5, 0.5, 2.0];
    let dir = [0.0, 0.0, -1.0];
    let hit = bsp.intersect_ray(&origin, &dir);
    assert_eq!(hit, vec![0]);
    let view_proj = matrix4f_identity();
    let frustum = Frustum::from_view_proj(&view_proj);
    let visible = bsp.intersect_frustum(&frustum);
    assert!(!visible.is_empty());
    assert!(visible.contains(&0));
}

#[test]
fn bvh_ray_and_frustum() {
    let items: Vec<(u32, Aabb)> = vec![
        (0, Aabb::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0])),
        (1, Aabb::new([2.0, 0.0, 0.0], [3.0, 1.0, 1.0])),
    ];
    let bvh = BvhTree::build(&items);
    let origin = [0.5, 0.5, 2.0];
    let dir = [0.0, 0.0, -1.0];
    let hit = bvh.intersect_ray(&origin, &dir);
    assert_eq!(hit.len(), 1);
    assert_eq!(hit[0], 0);
    let view_proj = matrix4f_identity();
    let frustum = Frustum::from_view_proj(&view_proj);
    let visible = bvh.intersect_frustum(&frustum);
    assert!(!visible.is_empty());
}

#[test]
fn ray_aabb_smoke() {
    let aabb = Aabb::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
    let t = ray_aabb(&[0.5, 0.5, 2.0], &[0.0, 0.0, -1.0], &aabb);
    assert!(t.is_some());
    assert!((t.unwrap() - 1.0).abs() < 1e-5);
}

#[test]
fn ray_sphere_smoke() {
    let s = Sphere::new([0.0, 0.0, 0.0], 1.0);
    let t = ray_sphere(&[0.0, 0.0, 2.0], &[0.0, 0.0, -1.0], &s);
    assert!(t.is_some());
    assert!((t.unwrap() - 1.0).abs() < 1e-5);
}

#[test]
fn ray_capsule_smoke() {
    let cap = Capsule::new([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], 0.5);
    let t = ray_capsule(&[0.5, 0.0, 2.0], &[0.0, 0.0, -1.0], &cap);
    assert!(t.is_some());
    assert!(t.unwrap() > 0.0);
}

#[test]
fn ray_obb_smoke() {
    let rot = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
    let obb = Obb::new([0.0, 0.0, 0.0], [0.5, 0.5, 0.5], rot);
    let t = ray_obb(&[0.0, 0.0, 2.0], &[0.0, 0.0, -1.0], &obb);
    assert!(t.is_some());
    assert!(t.unwrap() > 0.0 && t.unwrap() < 3.0);
}

#[test]
fn bvh_median_and_sah_same_result_set() {
    let items: Vec<(u32, Aabb)> = (0..20)
        .map(|i| {
            let x = (i % 5) as f32 * 2.0;
            let y = ((i / 5) % 4) as f32 * 2.0;
            let z = (i / 20) as f32;
            (i, Aabb::new([x, y, z], [x + 1.0, y + 1.0, z + 1.0]))
        })
        .collect();
    let tree_median = BvhTree::build_with_strategy(&items, BvhBuildStrategy::Median);
    let tree_sah = BvhTree::build_with_strategy(&items, BvhBuildStrategy::Sah);
    let tree_morton = BvhTree::build_with_strategy(&items, BvhBuildStrategy::Morton);
    let origin = [5.0, 5.0, 10.0];
    let dir = [0.0, 0.0, -1.0];
    let ray_median: HashSet<u32> = tree_median
        .intersect_ray(&origin, &dir)
        .into_iter()
        .collect();
    let ray_sah: HashSet<u32> = tree_sah.intersect_ray(&origin, &dir).into_iter().collect();
    let ray_morton: HashSet<u32> = tree_morton
        .intersect_ray(&origin, &dir)
        .into_iter()
        .collect();
    assert_eq!(
        ray_median, ray_sah,
        "ray hit set should match for Median vs Sah"
    );
    assert_eq!(
        ray_median, ray_morton,
        "ray hit set should match for Median vs Morton"
    );
    let view_proj = matrix4f_identity();
    let frustum = Frustum::from_view_proj(&view_proj);
    let frustum_median: HashSet<u32> = tree_median
        .intersect_frustum(&frustum)
        .into_iter()
        .collect();
    let frustum_sah: HashSet<u32> = tree_sah.intersect_frustum(&frustum).into_iter().collect();
    let frustum_morton: HashSet<u32> = tree_morton
        .intersect_frustum(&frustum)
        .into_iter()
        .collect();
    assert_eq!(
        frustum_median, frustum_sah,
        "frustum hit set should match for Median vs Sah"
    );
    assert_eq!(
        frustum_median, frustum_morton,
        "frustum hit set should match for Median vs Morton"
    );
}

#[test]
fn bvh_iter_same_result_as_vec() {
    let items: Vec<(u32, Aabb)> = vec![
        (0, Aabb::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0])),
        (1, Aabb::new([2.0, 0.0, 0.0], [3.0, 1.0, 1.0])),
        (2, Aabb::new([0.0, 2.0, 0.0], [1.0, 3.0, 1.0])),
    ];
    let tree = BvhTree::build(&items);
    let origin = [0.5, 0.5, 2.0];
    let dir = [0.0, 0.0, -1.0];
    let vec_hits: HashSet<u32> = tree.intersect_ray(&origin, &dir).into_iter().collect();
    let iter_hits: HashSet<u32> = tree.intersect_ray_iter(&origin, &dir).collect();
    assert_eq!(vec_hits, iter_hits);
    let view_proj = matrix4f_identity();
    let frustum = Frustum::from_view_proj(&view_proj);
    let vec_visible: HashSet<u32> = tree.intersect_frustum(&frustum).into_iter().collect();
    let iter_visible: HashSet<u32> = tree.intersect_frustum_iter(&frustum).collect();
    assert_eq!(vec_visible, iter_visible);
}
