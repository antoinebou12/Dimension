//! Benchmarks for line search (backtracking, Armijo, Wolfe).

use criterion::{Criterion, criterion_group};
use mathlib::{LineSearchOptions, armijo, backtracking, wolfe};
use std::hint::black_box;

fn quadratic_cost(x: &[f64]) -> f64 {
    x[0] * x[0] + x[1] * x[1]
}

pub fn bench_linesearch(c: &mut Criterion) {
    let x = [2.0_f64, 2.0];
    let d = [-1.0_f64, -1.0];
    let f = 8.0;
    let g_dot_d = -8.0;
    let opts = LineSearchOptions::default();
    let mut scratch = [0.0_f64; 2];

    c.bench_function("backtracking", |b| {
        b.iter(|| {
            let alpha = backtracking(
                black_box(&x),
                black_box(&d),
                black_box(f),
                black_box(g_dot_d),
                quadratic_cost,
                black_box(&opts),
                black_box(&mut scratch),
            );
            black_box(alpha)
        })
    });

    c.bench_function("armijo", |b| {
        b.iter(|| {
            let alpha = armijo(
                black_box(&x),
                black_box(&d),
                black_box(f),
                black_box(g_dot_d),
                quadratic_cost,
                black_box(&opts),
                black_box(&mut scratch),
            );
            black_box(alpha)
        })
    });

    let grad = |x: &[f64], g: &mut [f64]| {
        g[0] = 2.0 * x[0];
        g[1] = 2.0 * x[1];
    };
    let mut g_scratch = [0.0_f64; 2];
    c.bench_function("wolfe", |b| {
        b.iter(|| {
            let alpha = wolfe(
                black_box(&x),
                black_box(&d),
                black_box(f),
                black_box(g_dot_d),
                quadratic_cost,
                black_box(&grad),
                black_box(&opts),
                black_box(&mut scratch),
                black_box(&mut g_scratch),
            );
            black_box(alpha)
        })
    });
}

criterion_group!(benches, bench_linesearch);
