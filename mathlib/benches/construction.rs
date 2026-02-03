//! Benchmarks for sparse matrix construction from triplets.

use criterion::{BenchmarkId, Criterion, black_box, criterion_group};
use mathlib::{SparseMatrixCCS, SparseMatrixCDS, SparseMatrixCRS, SparseStorage};
use std::time::Duration;

#[path = "common.rs"]
mod common;
use common::{generate_banded_matrix, generate_diagonal_matrix, generate_random_sparse};

pub fn bench_construction(c: &mut Criterion) {
    let mut group = c.benchmark_group("construction");
    group.measurement_time(Duration::from_secs(8));
    group.sample_size(50);

    let sizes = vec![100, 1000, 5000];

    for &n in &sizes {
        let triplets = generate_random_sparse(n, 0.01);

        group.bench_with_input(BenchmarkId::new("CRS", n), &triplets, |b, t| {
            b.iter(|| {
                black_box(SparseMatrixCRS::from_triplets(
                    black_box(n),
                    black_box(n),
                    black_box(t),
                ))
            })
        });

        group.bench_with_input(BenchmarkId::new("CCS", n), &triplets, |b, t| {
            b.iter(|| {
                black_box(SparseMatrixCCS::from_triplets(
                    black_box(n),
                    black_box(n),
                    black_box(t),
                ))
            })
        });
    }

    // Diagonal matrix for CDS
    for &n in &[100, 1000, 5000] {
        let triplets = generate_diagonal_matrix(n);
        group.bench_with_input(BenchmarkId::new("CDS_diagonal", n), &triplets, |b, t| {
            b.iter(|| SparseMatrixCDS::from_triplets(black_box(n), black_box(n), black_box(t)))
        });
    }

    // Banded matrix for CDS
    for &n in &[100, 1000, 5000] {
        let triplets = generate_banded_matrix(n, 5);
        group.bench_with_input(BenchmarkId::new("CDS_banded", n), &triplets, |b, t| {
            b.iter(|| {
                black_box(SparseMatrixCDS::from_triplets(
                    black_box(n),
                    black_box(n),
                    black_box(t),
                ))
            })
        });
    }

    group.finish();
}

criterion_group!(benches, bench_construction);
