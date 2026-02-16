//! Benchmarks for Gauss-Newton.

use criterion::{Criterion, criterion_group};
use mathlib::{GaussNewtonOptions, gauss_newton};
use std::hint::black_box;

pub fn bench_gauss_newton(c: &mut Criterion) {
    let mut group = c.benchmark_group("gauss_newton");

    let x0 = vec![0.5_f64, 0.5];
    let xs = [0.0_f64, 1.0, 2.0, 3.0];
    let ys = [0.0_f64, 1.0, 2.0, 3.0];
    let residual = |x: &[f64]| {
        let (a, b) = (x[0], x[1]);
        xs.iter()
            .zip(ys.iter())
            .map(|(&xi, &yi)| yi - (a * xi + b))
            .collect::<Vec<_>>()
    };
    let jacobian = |x: &[f64]| {
        let _ = x;
        let mut j = mathlib::Matrix::with_storage(4, 2, mathlib::structure::Storage::Column);
        for (i, &xi) in xs.iter().enumerate() {
            j.set(i, 0, -xi);
            j.set(i, 1, -1.0);
        }
        j
    };
    let opts = GaussNewtonOptions::default();

    group.bench_function("linear_fit_4pts", |b| {
        b.iter(|| {
            let result = gauss_newton(
                black_box(&x0),
                black_box(&residual),
                black_box(&jacobian),
                black_box(&opts),
            )
            .unwrap();
            black_box(result)
        })
    });

    group.finish();
}

criterion_group!(benches, bench_gauss_newton);
