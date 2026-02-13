//! Criterion benchmarks for collision crate.

use collision::{
    convex_hull_2d, point_in_polygon_2d, ray_aabb, ray_capsule, ray_obb, ray_polygon_2d,
    ray_segment, ray_sphere, ray_triangle, Aabb, BspTree, BvhBuildStrategy, BvhTree, Capsule,
    Frustum, Obb, Sphere,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mathlib::cg::matrix4f_identity;

fn items_100() -> Vec<(u32, Aabb)> {
    (0..100)
        .map(|i| {
            let x = (i % 10) as f32;
            let y = ((i / 10) % 10) as f32;
            let z = (i / 100) as f32;
            (i, Aabb::new([x, y, z], [x + 1.0, y + 1.0, z + 1.0]))
        })
        .collect()
}

fn items_1000() -> Vec<(u32, Aabb)> {
    (0..1000)
        .map(|i| {
            let x = (i % 10) as f32;
            let y = ((i / 10) % 10) as f32;
            let z = (i / 100) as f32;
            (i, Aabb::new([x, y, z], [x + 1.0, y + 1.0, z + 1.0]))
        })
        .collect()
}

fn bench_ray_aabb(c: &mut Criterion) {
    let aabb = Aabb::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
    let origin = [0.5, 0.5, 2.0];
    let dir = [0.0, 0.0, -1.0];
    c.bench_function("ray_aabb", |b| {
        b.iter(|| {
            black_box(ray_aabb(
                black_box(&origin),
                black_box(&dir),
                black_box(&aabb),
            ))
        })
    });
}

fn bench_ray_triangle(c: &mut Criterion) {
    let origin = [0.0, 0.0, 1.0];
    let dir = [0.0, 0.0, -1.0];
    let v0 = [0.0, 0.0, 0.0];
    let v1 = [1.0, 0.0, 0.0];
    let v2 = [0.5, 1.0, 0.0];
    c.bench_function("ray_triangle", |b| {
        b.iter(|| {
            black_box(ray_triangle(
                black_box(&origin),
                black_box(&dir),
                black_box(&v0),
                black_box(&v1),
                black_box(&v2),
            ))
        })
    });
}

fn bench_aabb_union(c: &mut Criterion) {
    let a = Aabb::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
    let other = Aabb::new([2.0, 0.0, 0.0], [3.0, 1.0, 1.0]);
    c.bench_function("aabb_union", |b| {
        b.iter(|| black_box(a.union(black_box(&other))))
    });
}

fn bench_frustum_intersects_aabb(c: &mut Criterion) {
    let view_proj = matrix4f_identity();
    let frustum = Frustum::from_view_proj(&view_proj);
    let aabb = Aabb::new([-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]);
    c.bench_function("frustum_intersects_aabb", |b| {
        b.iter(|| black_box(frustum.intersects_aabb(black_box(&aabb))))
    });
}

fn bench_ray_sphere(c: &mut Criterion) {
    let sphere = Sphere::new([0.0, 0.0, 0.0], 1.0);
    let origin = [0.0, 0.0, 2.0];
    let dir = [0.0, 0.0, -1.0];
    c.bench_function("ray_sphere", |b| {
        b.iter(|| {
            black_box(ray_sphere(
                black_box(&origin),
                black_box(&dir),
                black_box(&sphere),
            ))
        })
    });
}

fn bench_ray_obb(c: &mut Criterion) {
    let rot = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
    let obb = Obb::new([0.0, 0.0, 0.0], [0.5, 0.5, 0.5], rot);
    let origin = [0.0, 0.0, 2.0];
    let dir = [0.0, 0.0, -1.0];
    c.bench_function("ray_obb", |b| {
        b.iter(|| {
            black_box(ray_obb(
                black_box(&origin),
                black_box(&dir),
                black_box(&obb),
            ))
        })
    });
}

fn bench_ray_capsule(c: &mut Criterion) {
    let cap = Capsule::new([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], 0.5);
    let origin = [0.5, 0.0, 2.0];
    let dir = [0.0, 0.0, -1.0];
    c.bench_function("ray_capsule", |b| {
        b.iter(|| {
            black_box(ray_capsule(
                black_box(&origin),
                black_box(&dir),
                black_box(&cap),
            ))
        })
    });
}

fn bench_sphere_aabb(c: &mut Criterion) {
    let sphere = Sphere::new([1.0, 2.0, 3.0], 0.5);
    c.bench_function("sphere_aabb", |b| b.iter(|| black_box(sphere.aabb())));
}

fn bench_sphere_intersects_aabb(c: &mut Criterion) {
    let sphere = Sphere::new([1.0, 1.0, 1.0], 1.0);
    let aabb = Aabb::new([0.0, 0.0, 0.0], [2.0, 2.0, 2.0]);
    c.bench_function("sphere_intersects_aabb", |b| {
        b.iter(|| black_box(sphere.intersects_aabb(black_box(&aabb))))
    });
}

fn bench_capsule_aabb(c: &mut Criterion) {
    let cap = Capsule::new([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], 0.5);
    c.bench_function("capsule_aabb", |b| b.iter(|| black_box(cap.aabb())));
}

fn bench_obb_aabb(c: &mut Criterion) {
    let rot = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
    let obb = Obb::new([1.0, 2.0, 3.0], [0.5, 0.5, 0.5], rot);
    c.bench_function("obb_aabb", |b| b.iter(|| black_box(obb.aabb())));
}

fn bench_aabb_union_many(c: &mut Criterion) {
    let aabbs: Vec<Aabb> = (0..100)
        .map(|i| {
            let x = (i % 10) as f32;
            let y = ((i / 10) % 10) as f32;
            let z = (i / 100) as f32;
            Aabb::new([x, y, z], [x + 0.5, y + 0.5, z + 0.5])
        })
        .collect();
    c.bench_function("aabb_union_many_100", |b| {
        b.iter(|| black_box(Aabb::union_many(aabbs.iter().copied())))
    });
}

fn bench_bsp_build(c: &mut Criterion) {
    let items = items_100();
    c.bench_function("bsp_build_100", |b| {
        b.iter(|| BspTree::build(black_box(&items)))
    });
}

fn bench_bsp_build_1000(c: &mut Criterion) {
    let items = items_1000();
    c.bench_function("bsp_build_1000", |b| {
        b.iter(|| BspTree::build(black_box(&items)))
    });
}

fn bench_bsp_intersect_ray(c: &mut Criterion) {
    let items = items_100();
    let tree = BspTree::build(&items);
    let origin = [5.0, 5.0, 10.0];
    let dir = [0.0, 0.0, -1.0];
    c.bench_function("bsp_intersect_ray_100", |b| {
        b.iter(|| black_box(tree.intersect_ray(black_box(&origin), black_box(&dir))))
    });
}

fn bench_bsp_intersect_ray_1000(c: &mut Criterion) {
    let items = items_1000();
    let tree = BspTree::build(&items);
    let origin = [5.0, 5.0, 10.0];
    let dir = [0.0, 0.0, -1.0];
    c.bench_function("bsp_intersect_ray_1000", |b| {
        b.iter(|| black_box(tree.intersect_ray(black_box(&origin), black_box(&dir))))
    });
}

fn bench_bvh_build(c: &mut Criterion) {
    let items = items_100();
    c.bench_function("bvh_build_100", |b| {
        b.iter(|| BvhTree::build(black_box(&items)))
    });
}

fn bench_bvh_build_100_sah(c: &mut Criterion) {
    let items = items_100();
    c.bench_function("bvh_build_100_sah", |b| {
        b.iter(|| BvhTree::build_with_strategy(black_box(&items), BvhBuildStrategy::Sah))
    });
}

fn bench_bvh_build_1000(c: &mut Criterion) {
    let items = items_1000();
    c.bench_function("bvh_build_1000", |b| {
        b.iter(|| BvhTree::build(black_box(&items)))
    });
}

fn bench_bvh_build_100_morton(c: &mut Criterion) {
    let items = items_100();
    c.bench_function("bvh_build_100_morton", |b| {
        b.iter(|| BvhTree::build_with_strategy(black_box(&items), BvhBuildStrategy::Morton))
    });
}

fn bench_bvh_build_1000_morton(c: &mut Criterion) {
    let items = items_1000();
    c.bench_function("bvh_build_1000_morton", |b| {
        b.iter(|| BvhTree::build_with_strategy(black_box(&items), BvhBuildStrategy::Morton))
    });
}

fn bench_bvh_intersect_ray(c: &mut Criterion) {
    let items = items_100();
    let tree = BvhTree::build(&items);
    let origin = [5.0, 5.0, 10.0];
    let dir = [0.0, 0.0, -1.0];
    c.bench_function("bvh_intersect_ray_100", |b| {
        b.iter(|| black_box(tree.intersect_ray(black_box(&origin), black_box(&dir))))
    });
}

fn bench_bvh_intersect_ray_100_sah(c: &mut Criterion) {
    let items = items_100();
    let tree = BvhTree::build_with_strategy(&items, BvhBuildStrategy::Sah);
    let origin = [5.0, 5.0, 10.0];
    let dir = [0.0, 0.0, -1.0];
    c.bench_function("bvh_intersect_ray_100_sah", |b| {
        b.iter(|| black_box(tree.intersect_ray(black_box(&origin), black_box(&dir))))
    });
}

fn bench_bvh_intersect_ray_1000(c: &mut Criterion) {
    let items = items_1000();
    let tree = BvhTree::build(&items);
    let origin = [5.0, 5.0, 10.0];
    let dir = [0.0, 0.0, -1.0];
    c.bench_function("bvh_intersect_ray_1000", |b| {
        b.iter(|| black_box(tree.intersect_ray(black_box(&origin), black_box(&dir))))
    });
}

fn bench_ray_segment(c: &mut Criterion) {
    let origin = [0.5, 0.5, 1.0];
    let dir = [0.0, 0.0, -1.0];
    let seg_a = [0.0, 0.0, 0.0];
    let seg_b = [1.0, 1.0, 0.0];
    c.bench_function("ray_segment", |b| {
        b.iter(|| {
            black_box(ray_segment(
                black_box(&origin),
                black_box(&dir),
                black_box(&seg_a),
                black_box(&seg_b),
            ))
        })
    });
}

fn bench_bsp_intersect_frustum(c: &mut Criterion) {
    let items = items_100();
    let tree = BspTree::build(&items);
    let view_proj = matrix4f_identity();
    let frustum = Frustum::from_view_proj(&view_proj);
    c.bench_function("bsp_intersect_frustum_100", |b| {
        b.iter(|| black_box(tree.intersect_frustum(black_box(&frustum))))
    });
}

fn bench_bvh_intersect_frustum(c: &mut Criterion) {
    let items = items_100();
    let tree = BvhTree::build(&items);
    let view_proj = matrix4f_identity();
    let frustum = Frustum::from_view_proj(&view_proj);
    c.bench_function("bvh_intersect_frustum_100", |b| {
        b.iter(|| black_box(tree.intersect_frustum(black_box(&frustum))))
    });
}

fn bench_convex_hull_2d(c: &mut Criterion) {
    let pts: Vec<[f32; 2]> = (0..100)
        .flat_map(|i| {
            let x = (i % 10) as f32;
            let y = (i / 10) as f32;
            [[x, y], [x + 0.5, y + 0.3]]
        })
        .collect();
    c.bench_function("convex_hull_2d_200", |b| {
        b.iter(|| black_box(convex_hull_2d(black_box(&pts))))
    });
}

fn bench_point_in_polygon_2d(c: &mut Criterion) {
    let polygon = [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]];
    let point = [5.0, 5.0];
    c.bench_function("point_in_polygon_2d", |b| {
        b.iter(|| black_box(point_in_polygon_2d(black_box(&point), black_box(&polygon))))
    });
}

fn bench_ray_polygon_2d(c: &mut Criterion) {
    let polygon = [[0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0]];
    let origin = [1.0, -1.0];
    let dir = [0.0, 1.0];
    c.bench_function("ray_polygon_2d", |b| {
        b.iter(|| {
            black_box(ray_polygon_2d(
                black_box(&origin),
                black_box(&dir),
                black_box(&polygon),
            ))
        })
    });
}

criterion_group!(
    benches,
    bench_ray_aabb,
    bench_ray_triangle,
    bench_ray_segment,
    bench_ray_sphere,
    bench_ray_obb,
    bench_ray_capsule,
    bench_aabb_union,
    bench_aabb_union_many,
    bench_frustum_intersects_aabb,
    bench_sphere_aabb,
    bench_sphere_intersects_aabb,
    bench_capsule_aabb,
    bench_obb_aabb,
    bench_bsp_build,
    bench_bsp_build_1000,
    bench_bsp_intersect_ray,
    bench_bsp_intersect_ray_1000,
    bench_bsp_intersect_frustum,
    bench_bvh_build,
    bench_bvh_build_100_sah,
    bench_bvh_build_100_morton,
    bench_bvh_build_1000,
    bench_bvh_build_1000_morton,
    bench_bvh_intersect_ray,
    bench_bvh_intersect_ray_100_sah,
    bench_bvh_intersect_ray_1000,
    bench_bvh_intersect_frustum,
    bench_convex_hull_2d,
    bench_point_in_polygon_2d,
    bench_ray_polygon_2d,
);
criterion_main!(benches);
