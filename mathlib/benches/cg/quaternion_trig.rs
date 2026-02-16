//! Benchmarks for quaternion (from_axis_angle, to_rotation_matrix4) and trig (sin, cos, atan2, degrees, radians).

use criterion::{Criterion, criterion_group};
use mathlib::{Quat4f, Vector, Vector3f, trig};
use std::hint::black_box;

fn axis_y() -> Vector3f {
    let mut v = Vector3f::with_capacity(3);
    v.set(0, 0.0);
    v.set(1, 1.0);
    v.set(2, 0.0);
    v
}

pub fn bench_quaternion(c: &mut Criterion) {
    let mut group = c.benchmark_group("quaternion");

    let axis = axis_y();
    let angle = 0.5f32;

    group.bench_function("from_axis_angle", |b| {
        b.iter(|| {
            let q = Quat4f::from_axis_angle(black_box(&axis), black_box(angle));
            black_box(q)
        });
    });

    group.bench_function("to_rotation_matrix4", |b| {
        let q = Quat4f::from_axis_angle(&axis, angle);
        b.iter(|| black_box(q.to_rotation_matrix4()));
    });

    group.bench_function("from_axis_angle_to_matrix4", |b| {
        b.iter(|| {
            let q = Quat4f::from_axis_angle(black_box(&axis), black_box(angle));
            black_box(q.to_rotation_matrix4())
        });
    });

    group.finish();
}

fn trig_input_vec(n: usize) -> Vector<f64> {
    let mut v = Vector::with_capacity(n);
    for i in 0..n {
        v.set(i, (i as f64) * 0.1);
    }
    v
}

pub fn bench_trig(c: &mut Criterion) {
    let mut group = c.benchmark_group("trig");
    let v = trig_input_vec(256);

    group.bench_function("sin", |b| b.iter(|| black_box(trig::sin(black_box(&v)))));
    group.bench_function("cos", |b| b.iter(|| black_box(trig::cos(black_box(&v)))));
    group.bench_function("degrees", |b| {
        b.iter(|| black_box(trig::degrees(black_box(&v))))
    });
    group.bench_function("radians", |b| {
        b.iter(|| black_box(trig::radians(black_box(&v))))
    });
    group.bench_function("atan2", |b| {
        let y = trig_input_vec(256);
        b.iter(|| black_box(trig::atan2(black_box(&y), black_box(&v))))
    });

    group.finish();
}

criterion_group!(benches, bench_quaternion, bench_trig);
