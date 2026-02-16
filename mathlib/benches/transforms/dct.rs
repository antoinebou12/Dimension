//! Benchmarks for DCT.

use criterion::{BenchmarkId, Criterion, criterion_group};
use mathlib::{dct2_forward, dct2_inverse};
use std::hint::black_box;

fn bench_dct(c: &mut Criterion) {
    let mut group = c.benchmark_group("dct");
    let sizes = [256, 1024, 4096, 16384];

    for &n in &sizes {
        let x: Vec<f64> = (0..n).map(|i| (i as f64 * 0.01).sin()).collect();
        group.bench_with_input(BenchmarkId::new("forward", n), &x, |b, x| {
            b.iter(|| dct2_forward(black_box(x)).unwrap())
        });
        let fwd = dct2_forward(&x).unwrap();
        group.bench_with_input(BenchmarkId::new("inverse", n), &fwd, |b, f| {
            b.iter(|| dct2_inverse(black_box(f)).unwrap())
        });
    }

    group.finish();
}

criterion_group!(benches, bench_dct);
