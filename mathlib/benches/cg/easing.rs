//! Benchmarks for easing (linear, sine, cubic, bounce, hermite, bspline) and quaternion slerp.

use criterion::{Criterion, criterion_group};
use mathlib::easing::{bspline, ease_in_out_cubic, ease_in_sine, ease_out_bounce, hermite, linear};
use mathlib::{Quat4f, Vector3f};
use std::hint::black_box;

fn axis_y() -> Vector3f {
    let mut v = Vector3f::with_capacity(3);
    v.set(0, 0.0);
    v.set(1, 1.0);
    v.set(2, 0.0);
    v
}

pub fn bench_easing(c: &mut Criterion) {
    let mut group = c.benchmark_group("easing");
    let t = 0.5f64;
    let pts: [f64; 4] = [0.0, 1.0, 2.0, 3.0];

    group.bench_function("linear", |b| b.iter(|| black_box(linear(black_box(t)))));
    group.bench_function("ease_in_sine", |b| {
        b.iter(|| black_box(ease_in_sine(black_box(t))))
    });
    group.bench_function("ease_in_out_cubic", |b| {
        b.iter(|| black_box(ease_in_out_cubic(black_box(t))))
    });
    group.bench_function("ease_out_bounce", |b| {
        b.iter(|| black_box(ease_out_bounce(black_box(t))))
    });
    group.bench_function("hermite", |b| {
        b.iter(|| black_box(hermite(0.0, 1.0, 0.0, 0.0, black_box(t))))
    });
    group.bench_function("bspline", |b| {
        b.iter(|| black_box(bspline(black_box(&pts), black_box(t))))
    });
    group.finish();

    let mut group = c.benchmark_group("easing_slerp");
    let axis = axis_y();
    let q0 = Quat4f::from_axis_angle(&axis, 0.0);
    let q1 = Quat4f::from_axis_angle(&axis, 1.0);
    let t32 = 0.5f32;

    group.bench_function("slerp", |b| {
        b.iter(|| black_box(q0.slerp(black_box(&q1), black_box(t32))))
    });
    group.finish();
}

criterion_group!(benches, bench_easing);
