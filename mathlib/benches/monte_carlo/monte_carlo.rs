//! Benchmarks for Monte Carlo (estimate_pi, integrate_1d).

use criterion::{Criterion, criterion_group};
use mathlib::{estimate_pi, integrate_1d};
use std::hint::black_box;

fn bench_monte_carlo(c: &mut Criterion) {
    let mut group = c.benchmark_group("monte_carlo");

    group.bench_function("estimate_pi_10k", |b| {
        b.iter(|| black_box(estimate_pi(black_box(42), black_box(10_000))))
    });
    group.bench_function("estimate_pi_100k", |b| {
        b.iter(|| black_box(estimate_pi(black_box(42), black_box(100_000))))
    });
    group.bench_function("estimate_pi_1M", |b| {
        b.iter(|| black_box(estimate_pi(black_box(42), black_box(1_000_000))))
    });

    group.bench_function("integrate_1d_10k", |b| {
        b.iter(|| black_box(integrate_1d(|x| x * x, 0.0, 1.0, black_box(10_000), 123)))
    });
    group.bench_function("integrate_1d_100k", |b| {
        b.iter(|| black_box(integrate_1d(|x| x * x, 0.0, 1.0, black_box(100_000), 123)))
    });

    group.finish();
}

criterion_group!(benches, bench_monte_carlo);
