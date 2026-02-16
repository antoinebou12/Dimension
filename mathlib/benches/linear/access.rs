//! Benchmarks for element access and format conversion.

use criterion::{BenchmarkId, Criterion, criterion_group};
use mathlib::{SparseMatrixCCS, SparseMatrixCRS, SparseStorage};
use std::hint::black_box;
use std::time::Duration;

#[path = "common.rs"]
mod common;
use common::generate_random_sparse;

pub fn bench_element_access(c: &mut Criterion) {
    let mut group = c.benchmark_group("element_access");

    let n = 1000;
    let triplets = generate_random_sparse(n, 0.01);

    let crs = SparseMatrixCRS::from_triplets(n, n, &triplets);
    group.bench_function("CRS_sequential", |b| {
        b.iter(|| {
            let mut sum = 0.0;
            for i in 0..100 {
                sum += crs.get(black_box(i), black_box(i));
            }
            black_box(sum)
        })
    });

    let ccs = SparseMatrixCCS::from_triplets(n, n, &triplets);
    group.bench_function("CCS_sequential", |b| {
        b.iter(|| {
            let mut sum = 0.0;
            for i in 0..100 {
                sum += ccs.get(black_box(i), black_box(i));
            }
            black_box(sum)
        })
    });

    group.bench_function("CRS_random", |b| {
        b.iter(|| {
            let mut sum = 0.0;
            for k in 0..100 {
                let i = (k * 7) % n;
                let j = (k * 13) % n;
                sum += crs.get(black_box(i), black_box(j));
            }
            black_box(sum)
        })
    });

    group.finish();
}

pub fn bench_conversion(c: &mut Criterion) {
    let mut group = c.benchmark_group("conversion");

    let n = 1000;
    let triplets = generate_random_sparse(n, 0.01);

    let crs = SparseMatrixCRS::from_triplets(n, n, &triplets);
    group.bench_function("CRS_to_triplets", |b| {
        b.iter(|| black_box(crs.to_triplets()))
    });

    group.finish();
}

pub fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    group.measurement_time(Duration::from_secs(8));
    group.sample_size(50);

    let sizes = vec![100, 1000, 5000];

    for &n in &sizes {
        let triplets = generate_random_sparse(n, 0.01);

        group.bench_with_input(BenchmarkId::new("CRS_alloc", n), &n, |b, &size| {
            b.iter(|| {
                black_box(SparseMatrixCRS::from_triplets(
                    black_box(size),
                    black_box(size),
                    black_box(&triplets),
                ))
            })
        });

        group.bench_with_input(BenchmarkId::new("CCS_alloc", n), &n, |b, &size| {
            b.iter(|| {
                black_box(SparseMatrixCCS::from_triplets(
                    black_box(size),
                    black_box(size),
                    black_box(&triplets),
                ))
            })
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_element_access,
    bench_conversion,
    bench_memory_usage
);
