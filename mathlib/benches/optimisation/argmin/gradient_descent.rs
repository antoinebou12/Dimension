//! Benchmarks for gradient descent.

use criterion::{BenchmarkId, Criterion, black_box, criterion_group};
use mathlib::{GradientDescentOptions, gradient_descent};

pub fn bench_gradient_descent(c: &mut Criterion) {
    let mut group = c.benchmark_group("gradient_descent");

    for dim in [4_usize, 16, 64] {
        let x0 = vec![1.0_f64; dim];
        let cost = |x: &[f64]| x.iter().map(|v| v * v).sum::<f64>();
        let gradient = |x: &[f64], g: &mut [f64]| {
            for (i, &xi) in x.iter().enumerate() {
                g[i] = 2.0 * xi;
            }
        };
        let opts = GradientDescentOptions {
            max_iters: 200,
            tol: 1e-10,
            ..Default::default()
        };

        group.bench_with_input(
            BenchmarkId::new("sphere", dim),
            &(x0.clone(), opts.clone()),
            |b, (x0, opts)| {
                b.iter(|| {
                    let result = gradient_descent(
                        black_box(x0.as_slice()),
                        black_box(&cost),
                        black_box(&gradient),
                        black_box(opts),
                    );
                    black_box(result)
                })
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_gradient_descent);
