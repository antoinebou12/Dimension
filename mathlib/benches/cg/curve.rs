//! Benchmarks for 3D curve evaluation (linear, Bezier, Hermite, B-spline).

use criterion::{Criterion, black_box, criterion_group};
use mathlib::math::curve::{bezier_curve, bspline_curve, hermite_curve, linear_curve};

pub fn benches(c: &mut Criterion) {
    let mut group = c.benchmark_group("curve");
    let p0 = [0.0_f32, 0.0, 0.0];
    let p1 = [1.0, 0.0, 0.0];
    let p2 = [1.0, 1.0, 0.0];
    let p3 = [1.0, 1.0, 1.0];
    let m0 = [0.0_f32, 0.0, 0.0];
    let m1 = [0.0_f32, 0.0, 0.0];
    let _t = 0.5_f32;

    group.bench_function("linear_curve_1000", |b| {
        b.iter(|| {
            for i in 0..1000 {
                let ti = (i as f32) / 1000.0;
                black_box(linear_curve(black_box(p0), black_box(p1), black_box(ti)));
            }
        })
    });
    group.bench_function("bezier_curve_1000", |b| {
        b.iter(|| {
            for i in 0..1000 {
                let ti = (i as f32) / 1000.0;
                black_box(bezier_curve(
                    black_box(p0),
                    black_box(p1),
                    black_box(p2),
                    black_box(p3),
                    black_box(ti),
                ));
            }
        })
    });
    group.bench_function("hermite_curve_1000", |b| {
        b.iter(|| {
            for i in 0..1000 {
                let ti = (i as f32) / 1000.0;
                black_box(hermite_curve(
                    black_box(p0),
                    black_box(p1),
                    black_box(m0),
                    black_box(m1),
                    black_box(ti),
                ));
            }
        })
    });
    group.bench_function("bspline_curve_1000", |b| {
        b.iter(|| {
            for i in 0..1000 {
                let ti = (i as f32) / 1000.0;
                black_box(bspline_curve(
                    black_box(p0),
                    black_box(p1),
                    black_box(p2),
                    black_box(p3),
                    black_box(ti),
                ));
            }
        })
    });
    group.finish();
}

criterion_group!(curve_benches, benches);
