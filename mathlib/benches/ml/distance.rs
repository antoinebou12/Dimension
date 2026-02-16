//! Benchmarks for distance metrics (euclidean_rows, cosine_similarity, minkowski).

use criterion::{BenchmarkId, Criterion, criterion_group};
use mathlib::{
    Matrix, Storage, Vector, cosine_similarity, euclidean_rows, minkowski, squared_euclidean_rows,
};
use std::hint::black_box;

fn synthetic_matrix(rows: usize, cols: usize) -> Matrix<f64> {
    let mut m = Matrix::with_storage(rows, cols, Storage::Column);
    for i in 0..rows {
        for j in 0..cols {
            m.set(i, j, (i as f64) * 0.13 + (j as f64) * 0.07);
        }
    }
    m
}

fn synthetic_vectors(n: usize) -> (Vector<f64>, Vector<f64>) {
    let mut a = Vector::with_capacity(n);
    let mut b = Vector::with_capacity(n);
    for i in 0..n {
        a.set(i, (i as f64) * 0.1);
        b.set(i, (i as f64) * 0.1 + 0.5);
    }
    (a, b)
}

pub fn bench_distance(c: &mut Criterion) {
    let mut group = c.benchmark_group("distance");

    let row_sizes = [(100, 10), (500, 20), (1000, 50)];
    for (rows, cols) in row_sizes {
        let data = synthetic_matrix(rows, cols);
        group.bench_with_input(
            BenchmarkId::new("euclidean_rows", format!("{}x{}", rows, cols)),
            &data,
            |b, d| b.iter(|| black_box(euclidean_rows(d, 0, rows - 1))),
        );
        group.bench_with_input(
            BenchmarkId::new("squared_euclidean_rows", format!("{}x{}", rows, cols)),
            &data,
            |b, d| b.iter(|| black_box(squared_euclidean_rows(d, 0, rows - 1))),
        );
    }

    let dims = [10, 100, 500];
    for n in dims {
        let (a, b) = synthetic_vectors(n);
        group.bench_with_input(
            BenchmarkId::new("cosine_similarity", n),
            &(&a, &b),
            |b, (u, v)| b.iter(|| black_box(cosine_similarity(u, v))),
        );
        group.bench_with_input(
            BenchmarkId::new("minkowski_p2", n),
            &(&a, &b),
            |b, (u, v)| b.iter(|| black_box(minkowski(u, v, 2.0))),
        );
    }

    group.finish();
}

criterion_group!(benches, bench_distance);
