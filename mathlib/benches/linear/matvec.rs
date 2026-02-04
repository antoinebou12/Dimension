//! Benchmarks for matrix-vector multiplication.

use criterion::{Criterion, black_box, criterion_group};
use mathlib::{SparseMatrixCCS, SparseMatrixCDS, SparseMatrixCRS, SparseStorage};

#[path = "common.rs"]
mod common;
use common::{
    generate_banded_matrix, generate_diagonal_matrix, generate_random_sparse, generate_small_dense,
    generate_vector,
};

pub fn bench_matvec(c: &mut Criterion) {
    let mut group = c.benchmark_group("matvec");

    // Small dense matrix
    {
        let n = 100;
        let triplets = generate_small_dense(n, 0.5);
        let x = generate_vector(n);

        let crs = SparseMatrixCRS::from_triplets(n, n, &triplets);
        group.bench_function("CRS_small_dense", |b| {
            b.iter(|| black_box(crs.mul_vector(black_box(&x))))
        });

        let ccs = SparseMatrixCCS::from_triplets(n, n, &triplets);
        group.bench_function("CCS_small_dense", |b| {
            b.iter(|| black_box(ccs.mul_vector(black_box(&x))))
        });
    }

    // Large sparse matrix
    {
        let n = 10000;
        let triplets = generate_random_sparse(n, 0.001);
        let x = generate_vector(n);

        let crs = SparseMatrixCRS::from_triplets(n, n, &triplets);
        group.bench_function("CRS_large_sparse", |b| {
            b.iter(|| black_box(crs.mul_vector(black_box(&x))))
        });

        let ccs = SparseMatrixCCS::from_triplets(n, n, &triplets);
        group.bench_function("CCS_large_sparse", |b| {
            b.iter(|| black_box(ccs.mul_vector(black_box(&x))))
        });
    }

    // Diagonal matrix
    {
        let n = 5000;
        let triplets = generate_diagonal_matrix(n);
        let x = generate_vector(n);

        let crs = SparseMatrixCRS::from_triplets(n, n, &triplets);
        group.bench_function("CRS_diagonal", |b| {
            b.iter(|| black_box(crs.mul_vector(black_box(&x))))
        });

        let cds = SparseMatrixCDS::from_triplets(n, n, &triplets);
        group.bench_function("CDS_diagonal", |b| {
            b.iter(|| black_box(cds.mul_vector(black_box(&x))))
        });
    }

    // Banded matrix
    {
        let n = 5000;
        let bandwidth = 10;
        let triplets = generate_banded_matrix(n, bandwidth);
        let x = generate_vector(n);

        let crs = SparseMatrixCRS::from_triplets(n, n, &triplets);
        group.bench_function("CRS_banded", |b| {
            b.iter(|| black_box(crs.mul_vector(black_box(&x))))
        });

        let cds = SparseMatrixCDS::from_triplets(n, n, &triplets);
        group.bench_function("CDS_banded", |b| {
            b.iter(|| black_box(cds.mul_vector(black_box(&x))))
        });
    }

    group.finish();
}

pub fn bench_matvec_transpose(c: &mut Criterion) {
    let mut group = c.benchmark_group("matvec_transpose");

    let n = 1000;
    let triplets = generate_random_sparse(n, 0.01);
    let x = generate_vector(n);

    let crs = SparseMatrixCRS::from_triplets(n, n, &triplets);
    group.bench_function("CRS", |b| {
        b.iter(|| black_box(crs.mul_vector_transpose(black_box(&x))))
    });

    let ccs = SparseMatrixCCS::from_triplets(n, n, &triplets);
    group.bench_function("CCS", |b| {
        b.iter(|| black_box(ccs.mul_vector_transpose(black_box(&x))))
    });

    // Diagonal for CDS
    let triplets_diag = generate_diagonal_matrix(n);
    let cds = SparseMatrixCDS::from_triplets(n, n, &triplets_diag);
    group.bench_function("CDS_diagonal", |b| {
        b.iter(|| black_box(cds.mul_vector_transpose(black_box(&x))))
    });

    group.finish();
}

criterion_group!(benches, bench_matvec, bench_matvec_transpose);
