//! Benchmarks for Schur decomposition.

use criterion::{Criterion, criterion_group};
use mathlib::{Matrix, Storage, schur};
use std::hint::black_box;

fn make_2x2() -> Matrix<f64> {
    let mut a = Matrix::with_storage(2, 2, Storage::Column);
    a.set_identity();
    a
}

pub fn bench_schur(c: &mut Criterion) {
    let mut group = c.benchmark_group("schur");
    let a = make_2x2();

    group.bench_function("schur_2x2", |bench| {
        bench.iter(|| black_box(schur(black_box(&a))))
    });

    group.finish();
}

criterion_group!(benches, bench_schur);
