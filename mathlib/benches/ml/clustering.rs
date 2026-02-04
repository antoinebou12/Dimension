//! Benchmarks for K-means and DBSCAN clustering.

use criterion::{BenchmarkId, Criterion, black_box, criterion_group};
use mathlib::{Matrix, Storage, dbscan, kmeans};
use std::time::Duration;

fn synthetic_data(n_samples: usize, n_features: usize) -> Matrix<f64> {
    let mut m = Matrix::with_storage(n_samples, n_features, Storage::Column);
    for i in 0..n_samples {
        for j in 0..n_features {
            m.set(i, j, (i as f64) * 0.13 + (j as f64) * 0.07 - 0.5);
        }
    }
    m
}

pub fn bench_clustering(c: &mut Criterion) {
    let mut group = c.benchmark_group("clustering");
    group.measurement_time(Duration::from_secs(8));
    group.sample_size(50);

    let sizes = [(100, 4), (500, 10), (1000, 8)];
    let k = 3;
    let max_iters = Some(50u32);

    for (n_samples, n_features) in sizes {
        let data = synthetic_data(n_samples, n_features);
        group.bench_with_input(
            BenchmarkId::new("kmeans", format!("{}x{}", n_samples, n_features)),
            &data,
            |b, d| b.iter(|| black_box(kmeans(black_box(d), k, max_iters))),
        );
    }

    for (n_samples, n_features) in sizes {
        let data = synthetic_data(n_samples, n_features);
        group.bench_with_input(
            BenchmarkId::new("dbscan", format!("{}x{}", n_samples, n_features)),
            &data,
            |b, d| b.iter(|| black_box(dbscan(black_box(d), 1.0, 3))),
        );
    }

    group.finish();
}

criterion_group!(benches, bench_clustering);
