//! Benchmarks for PCA (Principal Component Analysis).

use criterion::{BenchmarkId, Criterion, black_box, criterion_group};
use mathlib::{Matrix, Storage, pca};

fn random_matrix(rows: usize, cols: usize) -> Matrix<f64> {
    let mut m = Matrix::with_storage(rows, cols, Storage::Column);
    for i in 0..rows {
        for j in 0..cols {
            m.set(i, j, (i as f64) * 0.13 + (j as f64) * 0.07 - 0.5);
        }
    }
    m
}

pub fn bench_pca(c: &mut Criterion) {
    let mut group = c.benchmark_group("pca");

    let sizes = [(100, 10), (500, 20), (1000, 50), (2000, 30)];

    for (n_samples, n_features) in sizes {
        let data = random_matrix(n_samples, n_features);
        group.bench_with_input(
            BenchmarkId::new("full", format!("{}x{}", n_samples, n_features)),
            &data,
            |b, d| b.iter(|| pca(black_box(d), None)),
        );
        group.bench_with_input(
            BenchmarkId::new("k=5", format!("{}x{}", n_samples, n_features)),
            &data,
            |b, d| b.iter(|| black_box(pca(black_box(d), Some(5)))),
        );
    }

    group.finish();
}

criterion_group!(benches, bench_pca);
