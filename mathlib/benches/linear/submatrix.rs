//! Benchmarks for submatrix and block API.

use criterion::{BenchmarkId, Criterion, black_box, criterion_group};
use mathlib::Matrix;

#[path = "common.rs"]
mod common;

fn dense_matrix(n: usize) -> Matrix<f64> {
    let mut m = Matrix::with_dimensions(n, n);
    for i in 0..n {
        for j in 0..n {
            m.set(i, j, (i * n + j) as f64 + 1.0);
        }
    }
    m
}

pub fn bench_submatrix(c: &mut Criterion) {
    let mut group = c.benchmark_group("submatrix");

    let sizes = [64_usize, 256, 1024];

    for &n in &sizes {
        let half = n / 2;
        group.bench_with_input(BenchmarkId::new("block_create", n), &n, |b, &size| {
            b.iter(|| {
                let mut m = dense_matrix(size);
                let block = m.block(0, 0, half, half);
                black_box((block.rows(), block.cols()))
            })
        });
    }

    for &n in &sizes {
        let half = n / 2;
        group.bench_with_input(BenchmarkId::new("block_get_set", n), &n, |b, &size| {
            let mut m = dense_matrix(size);
            b.iter(|| {
                let mut block = m.block(0, 0, half, half);
                let mut sum = 0.0_f64;
                for i in 0..block.rows() {
                    for j in 0..block.cols() {
                        sum += block.get(i, j);
                        block.set(i, j, sum);
                    }
                }
                black_box(sum)
            })
        });
    }

    for &n in &sizes {
        let half = n / 2;
        group.bench_with_input(BenchmarkId::new("assign_from", n), &n, |b, &size| {
            let other = dense_matrix(half);
            b.iter(|| {
                let mut m = dense_matrix(size);
                let mut block = m.block(0, 0, half, half);
                block.assign_from(black_box(&other))
            })
        });
    }

    for &n in &sizes {
        let half = n / 2;
        group.bench_with_input(BenchmarkId::new("to_matrix", n), &n, |b, &size| {
            let mut m = dense_matrix(size);
            b.iter(|| {
                let block = m.block(0, 0, half, half);
                black_box(block.to_matrix())
            })
        });
    }

    for &n in &sizes {
        let half = n / 2;
        group.bench_with_input(BenchmarkId::new("transpose", n), &n, |b, &size| {
            let mut m = dense_matrix(size);
            b.iter(|| {
                let block = m.block(0, 0, half, half);
                black_box(block.transpose())
            })
        });
    }

    for &n in &sizes {
        let half = n / 2;
        group.bench_with_input(BenchmarkId::new("add_assign", n), &n, |b, &size| {
            let other = dense_matrix(half);
            b.iter(|| {
                let mut m = dense_matrix(size);
                let mut block = m.block(0, 0, half, half);
                block += black_box(&other)
            })
        });
    }

    group.finish();
}

criterion_group!(benches, bench_submatrix);
