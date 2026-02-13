//! Example: build BSP and BVH from AABBs, run frustum and ray queries.

use collision::{ray_aabb, Aabb, BspTree, BvhTree, Frustum};
use mathlib::cg::matrix4f_identity;

fn main() {
    let items: Vec<(u32, Aabb)> = vec![
        (0, Aabb::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0])),
        (1, Aabb::new([2.0, 0.0, 0.0], [3.0, 1.0, 1.0])),
        (2, Aabb::new([0.0, 2.0, 0.0], [1.0, 3.0, 1.0])),
        (3, Aabb::new([-5.0, -5.0, -5.0], [-4.0, -4.0, -4.0])),
    ];

    let bsp = BspTree::build(&items);
    let bvh = BvhTree::build(&items);

    let view_proj = matrix4f_identity();
    let frustum = Frustum::from_view_proj(&view_proj);

    let bsp_visible = bsp.intersect_frustum(&frustum);
    let bvh_visible = bvh.intersect_frustum(&frustum);

    println!("BSP frustum visible: {:?}", bsp_visible);
    println!("BVH frustum visible: {:?}", bvh_visible);

    let origin = [0.5, 0.5, 2.0];
    let dir = [0.0, 0.0, -1.0];
    let bsp_hit = bsp.intersect_ray(&origin, &dir);
    let bvh_hit = bvh.intersect_ray(&origin, &dir);

    println!("BSP ray hit: {:?}", bsp_hit);
    println!("BVH ray hit: {:?}", bvh_hit);

    let t = ray_aabb(&origin, &dir, &items[0].1);
    println!("Direct ray_aabb t: {:?}", t);
}
