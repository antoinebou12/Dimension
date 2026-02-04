//! Benchmarks for general linear solve Ax = b.

use criterion::{Criterion, black_box, criterion_group};
use mathlib::{Matrix, Storage, Vector, solve};

fn make_square(n: usize) -> Matrix<f64> {
    let mut a = Matrix::with_storage(n, n, Storage::Column);
    for i in 0..n {
        for j in 0..n {
            let v = if i == j { 2.0 + (i as f64) * 0.1 } else { 0.1 };
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

pub fn bench_solve(c: &mut Criterion) {
    let mut group = c.benchmark_group("solve");
    let a = make_square(64);
    let b = make_rhs(64);

    group.bench_function("solve_64x64", |bench| {
        bench.iter(|| black_box(solve(black_box(&a), black_box(&b)).unwrap()))
    });

    group.finish();
}

criterion_group!(benches, bench_solve);
