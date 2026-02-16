//! Benchmarks for QZ (generalised Schur) decomposition.

use criterion::{Criterion, criterion_group};
use mathlib::{Matrix, Storage, qz};
use std::hint::black_box;

fn make_2x2() -> (Matrix<f64>, Matrix<f64>) {
    let mut a = Matrix::with_storage(2, 2, Storage::Column);
    let mut b = Matrix::with_storage(2, 2, Storage::Column);
    a.set_identity();
    b.set_identity();
    (a, b)
}

pub fn bench_qz(c: &mut Criterion) {
    let mut group = c.benchmark_group("qz");
    let (a, b) = make_2x2();

    group.bench_function("qz_2x2", |bench| {
        bench.iter(|| black_box(qz(black_box(&a), black_box(&b))))
    });

    group.finish();
}

criterion_group!(benches, bench_qz);
