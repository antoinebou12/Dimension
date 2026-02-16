//! Benchmarks for Cholesky decomposition and solve.

use criterion::{Criterion, criterion_group};
use mathlib::{Cholesky, Matrix, Storage, Vector};
use std::hint::black_box;

/// Symmetric positive definite: diagonal dominant with small off-diagonal.
fn make_spd(n: usize) -> Matrix<f64> {
    let mut a = Matrix::with_storage(n, n, Storage::Column);
    for i in 0..n {
        for j in 0..n {
            let v = if i == j { 4.0 + (i as f64) * 0.2 } else { 0.1 };
            a.set(i, j, v);
        }
    }
    a
}

fn make_rhs(n: usize) -> Vector<f64> {
    let mut b = Vector::with_capacity(n);
    for i in 0..n {
        b.set(i, (i + 1) as f64);
    }
    b
}

pub fn bench_chol(c: &mut Criterion) {
    let mut group = c.benchmark_group("chol");
    let a = make_spd(64);
    let b = make_rhs(64);

    group.bench_function("Cholesky::new_64x64", |bench| {
        bench.iter(|| black_box(Cholesky::new(black_box(&a)).unwrap()))
    });
    group.bench_function("chol_solve_64x64", |bench| {
        let ch = Cholesky::new(&a).unwrap();
        bench.iter(|| black_box(ch.solve(black_box(&b))))
    });

    group.finish();
}

criterion_group!(benches, bench_chol);
