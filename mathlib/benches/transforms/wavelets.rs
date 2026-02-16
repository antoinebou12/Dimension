//! Benchmarks for Haar wavelets.

use criterion::{BenchmarkId, Criterion, criterion_group};
use mathlib::{dwt_haar_forward, dwt_haar_inverse};
use std::hint::black_box;

fn bench_wavelets(c: &mut Criterion) {
    let mut group = c.benchmark_group("wavelets_haar");
    let sizes = [256, 1024, 4096, 16384];

    for &n in &sizes {
        let x: Vec<f64> = (0..n).map(|i| (i as f64 * 0.01).sin()).collect();
        group.bench_with_input(BenchmarkId::new("forward", n), &x, |b, x| {
            b.iter(|| dwt_haar_forward(black_box(x)))
        });
        let fwd = dwt_haar_forward(&x);
        group.bench_with_input(BenchmarkId::new("inverse", n), &fwd, |b, f| {
            b.iter(|| dwt_haar_inverse(black_box(f)))
        });
    }

    group.finish();
}

criterion_group!(benches, bench_wavelets);
