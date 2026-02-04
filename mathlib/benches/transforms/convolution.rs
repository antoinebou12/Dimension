//! Benchmarks for convolution.

use criterion::{BenchmarkId, Criterion, black_box, criterion_group};
use mathlib::{Matrix, Storage, conv_1d, conv_1d_same, conv_2d};

fn bench_convolution(c: &mut Criterion) {
    let mut group = c.benchmark_group("convolution");

    let signal_sizes = [256, 1024, 4096];
    let kernel_sizes = [7, 31];

    for &sl in &signal_sizes {
        for &kl in &kernel_sizes {
            let signal: Vec<f64> = (0..sl).map(|i| (i as f64 * 0.01).sin()).collect();
            let kernel: Vec<f64> = vec![1.0 / kl as f64; kl];
            group.bench_with_input(
                BenchmarkId::new("conv_1d", format!("{}x{}", sl, kl)),
                &(&signal, &kernel),
                |b, (sig, k)| b.iter(|| conv_1d(black_box(sig), black_box(k))),
            );
            group.bench_with_input(
                BenchmarkId::new("conv_1d_same", format!("{}x{}", sl, kl)),
                &(&signal, &kernel),
                |b, (sig, k)| b.iter(|| conv_1d_same(black_box(sig), black_box(k))),
            );
        }
    }

    let mat_sizes = [(64, 64), (128, 128), (256, 256)];
    for (rows, cols) in mat_sizes {
        let mut m = Matrix::with_storage(rows, cols, Storage::Column);
        for i in 0..rows {
            for j in 0..cols {
                m.set(i, j, (i * cols + j) as f64 * 0.01);
            }
        }
        let mut k = Matrix::with_storage(5, 5, Storage::Column);
        k.set_zero();
        for i in 0..5 {
            for j in 0..5 {
                k.set(i, j, 1.0 / 25.0);
            }
        }
        group.bench_with_input(
            BenchmarkId::new("conv_2d", format!("{}x{}", rows, cols)),
            &(&m, &k),
            |b, (mat, ker)| b.iter(|| conv_2d(black_box(mat), black_box(ker))),
        );
    }

    group.finish();
}

criterion_group!(benches, bench_convolution);
