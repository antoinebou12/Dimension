//! Benchmarks for simplex (linear programming) solver.

use criterion::{Criterion, black_box, criterion_group};
use mathlib::{Matrix, Storage, Vector, simplex_solve};

/// Build a small feasible LP: min c'x s.t. Ax = b, x >= 0 (standard form).
/// Returns (c, A, b) with A m×n.
fn make_simple_lp(m: usize, n: usize) -> (Vector<f64>, Matrix<f64>, Vector<f64>) {
    let mut c = Vector::with_capacity(n);
    for j in 0..n {
        c.set(j, -1.0 / (j + 1) as f64);
    }
    let mut a = Matrix::with_storage(m, n, Storage::Row);
    a.set_zero();
    for i in 0..m {
        a.set(i, i % n, 1.0);
        if n > 1 {
            a.set(i, (i + 1) % n, 1.0);
        }
    }
    let mut b = Vector::with_capacity(m);
    for i in 0..m {
        b.set(i, 1.0);
    }
    (c, a, b)
}

pub fn bench_simplex(c: &mut Criterion) {
    let mut group = c.benchmark_group("simplex");

    let (c, a, b) = make_simple_lp(1, 2);
    group.bench_function("simplex_1x2", |bench| {
        bench.iter(|| black_box(simplex_solve(black_box(&c), black_box(&a), black_box(&b))))
    });

    let (c, a, b) = make_simple_lp(2, 4);
    group.bench_function("simplex_2x4", |bench| {
        bench.iter(|| black_box(simplex_solve(black_box(&c), black_box(&a), black_box(&b))))
    });

    group.finish();
}

criterion_group!(benches, bench_simplex);
