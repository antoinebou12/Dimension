//! Benchmarks for dual quaternion (compose, transform_point, to_matrix4).

use criterion::{Criterion, black_box, criterion_group};
use mathlib::cg::vector3;
use mathlib::{DualQuat4f, Quat4f};

pub fn bench_dual_quaternion(c: &mut Criterion) {
    let mut group = c.benchmark_group("dual_quaternion");

    let rot = Quat4f::from_axis_angle(&vector3(0.0, 1.0, 0.0), 0.3);
    let t = vector3(1.0, 0.0, 0.0);
    let dq = DualQuat4f::from_rotation_and_translation(&rot, &t);
    let p = vector3(1.0, 0.0, 0.0);

    group.bench_function("transform_point", |b| {
        b.iter(|| black_box(dq.transform_point(black_box(&p))));
    });

    group.bench_function("to_matrix4", |b| {
        b.iter(|| black_box(dq.to_matrix4()));
    });

    let dq2 = DualQuat4f::from_rotation_and_translation(
        &Quat4f::from_axis_angle(&vector3(1.0, 0.0, 0.0), 0.1),
        &vector3(0.0, 1.0, 0.0),
    );
    group.bench_function("mul_compose", |b| {
        b.iter(|| black_box(dq * dq2));
    });

    group.finish();
}

criterion_group!(benches, bench_dual_quaternion);
