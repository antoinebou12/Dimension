//! Benchmark: mathlib SIMD vs scalar for matrix prep; render frame prep (CPU-side).

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mathlib::cg::{
    from_euler_angles, matrix4f_identity, new_nonuniform_scaling, new_translation, vector3,
};
use mathlib::math3d::Matrix4f;
use render::backend::math_prep::world_matrix;
use render::backend::primitive_mesh;
use render::gizmo::{gizmo_mesh, GizmoMode};
use render::pick_entity;
use render::scene::{CurvePoint, Primitive, Primitive2D, Primitive3D};
use render::{Camera, World};

fn bench_model_matrix_scalar(c: &mut Criterion) {
    c.bench_function("model_matrix_scalar", |b| {
        b.iter(|| {
            let pos = vector3(1.0, 2.0, 0.0);
            let scale = vector3(1.0, 1.0, 1.0);
            let t = new_translation(&pos);
            let r = from_euler_angles(0.0, 0.0, 0.5);
            let s = new_nonuniform_scaling(&scale);
            let m = &(&t * &r) * &s;
            black_box(m);
        })
    });
}

/// Build a world with n entities in a chain, each with a primitive.
fn world_with_n_entities(n: usize) -> World {
    let mut world = World::new();
    let root = world.root_entity();
    let prims: [Primitive; 3] = [
        Primitive::TwoD(Primitive2D::Quad),
        Primitive::ThreeD(Primitive3D::Cube),
        Primitive::ThreeD(Primitive3D::Cylinder),
    ];
    let mut parent = root;
    for i in 0..n {
        let e = world.spawn(parent);
        world.set_primitive(e, prims[i % 3]);
        parent = e;
    }
    world
}

fn bench_render_frame_prep_100(c: &mut Criterion) {
    c.bench_function("render_frame_prep_100", |b| {
        let world = world_with_n_entities(100);
        let tree = world.tree();
        b.iter(|| {
            let entities = world.entities_dfs();
            let n = tree.num_nodes();
            let mut world_matrices: Vec<Matrix4f> = (0..n).map(|_| matrix4f_identity()).collect();
            for id in &entities {
                let parent_world = world
                    .parent(*id)
                    .map(|p| world_matrices[p.0].clone())
                    .unwrap_or_else(matrix4f_identity);
                world_matrices[id.0] = world_matrix(tree, id.0, &parent_world);
            }
            let primitives: Vec<_> = entities
                .iter()
                .filter_map(|id| world.get(*id).and_then(|node| node.primitive))
                .collect();
            black_box((world_matrices, primitives));
        })
    });
}

fn bench_pick_entity_100(c: &mut Criterion) {
    let world = world_with_n_entities(100);
    let camera = Camera::new_perspective(800.0, 600.0, std::f32::consts::FRAC_PI_4);
    c.bench_function("pick_entity_100", |b| {
        b.iter(|| {
            let _ = pick_entity(
                black_box(&world),
                black_box(&camera),
                black_box(400.0),
                black_box(300.0),
            );
        })
    });
}

fn bench_gizmo_mesh_translate(c: &mut Criterion) {
    c.bench_function("gizmo_mesh_translate", |b| {
        b.iter(|| {
            let (v, i) = gizmo_mesh(black_box(GizmoMode::Translate), black_box(0.5));
            black_box((v, i));
        })
    });
}

fn bench_curve_mesh_all(c: &mut Criterion) {
    let line = Primitive::ThreeD(Primitive3D::LineSegment {
        start: CurvePoint([0.0, 0.0, 0.0]),
        end: CurvePoint([1.0, 0.0, 0.0]),
    });
    let bezier = Primitive::ThreeD(Primitive3D::Bezier {
        control_points: [
            CurvePoint([0.0, 0.0, 0.0]),
            CurvePoint([0.33, 0.0, 0.0]),
            CurvePoint([0.66, 0.0, 0.0]),
            CurvePoint([1.0, 0.0, 0.0]),
        ],
    });
    c.bench_function("curve_mesh_line_bezier_hermite_bspline", |b| {
        b.iter(|| {
            let _ = primitive_mesh(black_box(&line));
            let _ = primitive_mesh(black_box(&bezier));
            let herm = Primitive::ThreeD(Primitive3D::Hermite {
                p0: CurvePoint([0.0, 0.0, 0.0]),
                p1: CurvePoint([1.0, 0.0, 0.0]),
                m0: CurvePoint([0.0, 0.0, 0.0]),
                m1: CurvePoint([0.0, 0.0, 0.0]),
            });
            let _ = primitive_mesh(&herm);
            let bs = Primitive::ThreeD(Primitive3D::BSpline {
                control_points: [
                    CurvePoint([0.0, 0.0, 0.0]),
                    CurvePoint([0.33, 0.0, 0.0]),
                    CurvePoint([0.66, 0.0, 0.0]),
                    CurvePoint([1.0, 0.0, 0.0]),
                ],
            });
            let _ = primitive_mesh(&bs);
        })
    });
}

criterion_group!(
    benches,
    bench_model_matrix_scalar,
    bench_render_frame_prep_100,
    bench_pick_entity_100,
    bench_gizmo_mesh_translate,
    bench_curve_mesh_all
);
criterion_main!(benches);
