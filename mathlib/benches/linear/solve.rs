//! Benchmarks for general linear solve Ax = b and damped least-squares.

use criterion::{Criterion, black_box, criterion_group};
use mathlib::{Matrix, Storage, Vector, damped_least_squares, solve};

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

fn make_rect(m: usize, n: usize) -> Matrix<f64> {
    let mut a = Matrix::with_storage(m, n, Storage::Column);
    for i in 0..m {
        for j in 0..n {
            a.set(i, j, if i == j { 1.0 + (i as f64) * 0.1 } else { 0.05 });
        }
    }
    a
}

pub fn bench_solve(c: &mut Criterion) {
    let mut group = c.benchmark_group("solve");
    for n in [64_usize, 128, 256] {
        let a = make_square(n);
        let b = make_rhs(n);
        group.bench_function(format!("solve_{n}x{n}"), |bench| {
            bench.iter(|| black_box(solve(black_box(&a), black_box(&b)).unwrap()))
        });
    }
    group.finish();
}

pub fn bench_damped_least_squares(c: &mut Criterion) {
    let lambda_sq = 0.01;
    let mut group = c.benchmark_group("damped_least_squares");
    for (m, n) in [(64_usize, 32_usize), (128, 64), (256, 128)] {
        let a = make_rect(m, n);
        let b = make_rhs(m);
        group.bench_function(format!("damped_least_squares_{m}x{n}"), |bench| {
            bench.iter(|| {
                black_box(
                    damped_least_squares(black_box(&a), black_box(&b), black_box(lambda_sq))
                        .unwrap(),
                )
            })
        });
    }
    group.finish();
}

criterion_group!(benches, bench_solve, bench_damped_least_squares);
