# collision — Spatial queries and culling

**collision** is a top-level crate in the Dimension repo for collision detection and spatial queries: AABB (3D and 2D), ray casting (ray–AABB, ray–triangle, ray–segment, ray–sphere, ray–OBB, ray–capsule; 2D: ray_aabb_2d, ray_circle_2d), view frustum extraction and AABB culling, point-cloud AABB and bounding sphere, world_aabb transform, and spatial acceleration structures (BSP tree, BVH). Optional shapes: sphere, OBB, capsule; 2D: Aabb2, Circle, convex_hull_2d, point_in_polygon_2d.

For project context see [AGENTS.md](../AGENTS.md). Implementation plan: [collision-crate-plan.md](collision-crate-plan.md).

## Layout

| Path | Role |
|------|------|
| `collision/src/aabb.rs` | AABB (min/max), corners, union, center, half_extents, expand; ray_aabb (slab method; SIMD when feature `simd`). |
| `collision/src/aabb2.rs` | Aabb2 (2D min/max), from_center_half_extents, corners, union, intersects, contains, expand; ray_aabb_2d. |
| `collision/src/ray.rs` | ray_triangle (Möller–Trumbore), ray_segment; re-exports ray_aabb. |
| `collision/src/frustum.rs` | Frustum from view–projection matrix; intersects_aabb (positive-vertex test; SIMD when `simd`). |
| `collision/src/bsp.rs` | BSP tree over `(T, Aabb)`; build, intersect_frustum, intersect_ray. |
| `collision/src/bvh.rs` | BVH over `(T, Aabb)`; build (median, SAH, or Morton via `BvhBuildStrategy`), build_with_strategy, intersect_frustum, intersect_ray, intersect_frustum_iter, intersect_ray_iter. |
| `collision/src/sphere.rs` | Sphere (center, radius); aabb, intersects_aabb, ray_intersect. |
| `collision/src/obb.rs` | OBB (center, half-extents, rotation); aabb, ray_intersect. |
| `collision/src/capsule.rs` | Capsule (segment + radius); aabb, ray_intersect; ray_capsule. |
| `collision/src/circle.rs` | Circle (2D center, radius); aabb, contains_point, ray_intersect; ray_circle_2d. |
| `collision/src/convex2d.rs` | convex_hull_2d (Graham scan), point_in_polygon_2d (ray-cast), ray_polygon_2d (convex). |
| `collision/src/point_cloud.rs` | point_cloud_aabb, local_point_cloud_aabb, world_aabb, point_cloud_bounding_sphere (uses mathlib transform_point). |

## Dependencies

- **mathlib** — For `Matrix4f`, `matrix4f_to_array` (frustum), `transform_point` / `vector3` (point-cloud AABB and world_aabb), and optional `simd` / `parallel` features.
- **render** depends on **collision** for culling and picking (Aabb, Frustum, ray_*, BspTree).

## Features

| Feature | Purpose |
|---------|---------|
| `default` | Scalar only. |
| `simd` | SIMD paths (AABB union, ray_aabb slab, Aabb2 union, frustum `intersects_aabb`, point-cloud min/max with `wide`). |
| `parallel` | Parallel BVH build via par-core/chili; enables mathlib/parallel. Not on `wasm32` (build script rejects). `BvhTree<T>` requires `T: Send + Sync` when this feature is enabled. |

## BSP vs BVH

- **BSP**: Axis-aligned splits (X, Y, Z by depth). Good for static or infrequently updated data; predictable depth. Use when you want simple, deterministic traversal.
- **BVH**: Build with median split (default), binned SAH, or Morton (LBVH) via [`BvhBuildStrategy`]. Median is fast; SAH gives better tree quality; Morton uses a single global sort by Z-order code for fast, cache-friendly build and good spatial coherence. Use when you want a balance of build speed and query performance.

Both support `intersect_frustum` and `intersect_ray` over `(T, Aabb)` items. The BVH also provides `intersect_frustum_iter` and `intersect_ray_iter` for allocation-free iteration. The **render** crate uses BSP for picking and can use either for frustum culling.

## Usage

```rust
use collision::{Aabb, BspTree, Frustum, ray_aabb};
use mathlib::cg::matrix4f_identity;

// Ray–AABB
let aabb = Aabb::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
let t = ray_aabb(&[0.5, 0.5, 2.0], &[0.0, 0.0, -1.0], &aabb);

// Frustum
let view_proj = matrix4f_identity();
let frustum = Frustum::from_view_proj(&view_proj);
let visible = frustum.intersects_aabb(&aabb);

// BSP
let items = vec![(0u32, aabb), (1u32, other_aabb)];
let tree = BspTree::build(&items);
let hit = tree.intersect_ray(&origin, &dir);
let visible = tree.intersect_frustum(&frustum);

// BVH with SAH or iterator (no Vec allocation)
use collision::{BvhBuildStrategy, BvhTree};
let bvh = BvhTree::build_with_strategy(&items, BvhBuildStrategy::Sah);
let _first_hit = bvh.intersect_ray_iter(&origin, &dir).next();
```

## Tests and benchmarks

- **Tests**: `cargo test -p collision` (unit tests in each module; integration tests `tests/bsp_bvh_queries.rs`, `tests/convex_hull_point_in_poly.rs`).
- **Benchmarks**: `cargo bench -p collision` (ray_aabb, aabb_union, frustum_intersects_aabb, bsp/bvh build and intersect, convex_hull_2d, point_in_polygon_2d).
- **Examples**: `cargo run -p collision --example frustum_culling`, `cargo run -p collision --example convex_hull2d`.

## Build

From repo root:

- `cargo build -p collision`
- `cargo build -p collision --features simd`
- `cargo test -p collision`
- `cargo bench -p collision`

See [justfile](../justfile) for `build-collision`, `test-collision`, `bench-collision` (and optional `build-collision-simd`).
