//! Benchmarks for spectral windows.

use criterion::{BenchmarkId, Criterion, black_box, criterion_group};
use mathlib::{blackman, hamming, hann, tukey};

fn bench_windows(c: &mut Criterion) {
    let mut group = c.benchmark_group("windows");
    let sizes = [256, 1024, 4096, 16384];

    for &n in &sizes {
        group.bench_with_input(BenchmarkId::new("hann", n), &n, |b, &n| {
            b.iter(|| hann(black_box(n)))
        });
        group.bench_with_input(BenchmarkId::new("hamming", n), &n, |b, &n| {
            b.iter(|| hamming(black_box(n)))
        });
        group.bench_with_input(BenchmarkId::new("blackman", n), &n, |b, &n| {
            b.iter(|| blackman(black_box(n)))
        });
        group.bench_with_input(BenchmarkId::new("tukey_0.5", n), &n, |b, &n| {
            b.iter(|| tukey(black_box(n), 0.5))
        });
    }

    group.finish();
}

criterion_group!(benches, bench_windows);
