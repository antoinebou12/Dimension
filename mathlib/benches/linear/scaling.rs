//! Benchmarks for scaling behavior across different matrix sizes.

use criterion::{BenchmarkId, Criterion, black_box, criterion_group};
use mathlib::{SparseMatrixCRS, SparseStorage};

#[path = "common.rs"]
mod common;
use common::{generate_random_sparse, generate_vector};

pub fn bench_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("scaling");

    let sizes = vec![100, 500, 1000, 2000, 5000];

    for &n in &sizes {
        let triplets = generate_random_sparse(n, 0.01);
        let x = generate_vector(n);
        let crs = SparseMatrixCRS::from_triplets(n, n, &triplets);

        group.bench_with_input(BenchmarkId::new("CRS_matvec", n), &x, |b, vec| {
            b.iter(|| black_box(crs.mul_vector(black_box(vec))))
        });
    }

    group.finish();
}

criterion_group!(benches, bench_scaling);
