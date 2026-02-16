//! Benchmarks for FFT.

use criterion::{BenchmarkId, Criterion, criterion_group};
use mathlib::{Complex64, fft_forward, fft_forward_real, fft_inverse};
use std::hint::black_box;

fn bench_fft(c: &mut Criterion) {
    let mut group = c.benchmark_group("fft");
    let sizes = [256, 1024, 4096, 16384];

    for &n in &sizes {
        let x: Vec<Complex64> = (0..n)
            .map(|i| Complex64::new((i as f64 * 0.01).cos(), (i as f64 * 0.01).sin()))
            .collect();
        group.bench_with_input(BenchmarkId::new("forward", n), &x, |b, x| {
            b.iter(|| fft_forward(black_box(x)).unwrap())
        });
        let fwd = fft_forward(&x).unwrap();
        group.bench_with_input(BenchmarkId::new("inverse", n), &fwd, |b, f| {
            b.iter(|| fft_inverse(black_box(f)).unwrap())
        });
    }

    for &n in &sizes {
        let x: Vec<f64> = (0..n).map(|i| (i as f64 * 0.01).sin()).collect();
        group.bench_with_input(BenchmarkId::new("forward_real", n), &x, |b, x| {
            b.iter(|| fft_forward_real(black_box(x)).unwrap())
        });
    }

    group.finish();
}

criterion_group!(benches, bench_fft);
