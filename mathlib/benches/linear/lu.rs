//! Benchmarks for LU decomposition and solve.

use criterion::{Criterion, black_box, criterion_group};
use mathlib::{Lu, Matrix, Storage, Vector};

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

pub fn bench_lu(c: &mut Criterion) {
    let mut group = c.benchmark_group("lu");
    let a = make_square(64);
    let b = make_rhs(64);

    group.bench_function("Lu::new_64x64", |bench| {
        bench.iter(|| black_box(Lu::new(black_box(&a)).unwrap()))
    });
    group.bench_function("lu_solve_64x64", |bench| {
        let lu = Lu::new(&a).unwrap();
        bench.iter(|| black_box(lu.solve(black_box(&b))))
    });

    group.finish();
}

criterion_group!(benches, bench_lu);
