//! Benchmarks for CMA-ES (sphere function). Requires `genetic` feature.

#![cfg(feature = "genetic")]

use criterion::{Criterion, black_box, criterion_group};
use mathlib::CmaEsBuilder;

/// Sphere: minimize Σ xᵢ²; optimum at 0.
fn sphere(x: &[f64]) -> f64 {
    x.iter().map(|&v| v * v).sum()
}

fn bench_cmaes_sphere(c: &mut Criterion) {
    let mut group = c.benchmark_group("cmaes");

    for dim in [4, 10, 20] {
        group.bench_with_input(
            criterion::BenchmarkId::new("sphere", dim),
            &dim,
            |b, &dim| {
                b.iter(|| {
                    let mean = vec![1.0; dim];
                    let mut opt = CmaEsBuilder::new(dim, mean, 0.3)
                        .max_generations(50)
                        .seed(42)
                        .build();
                    let result = opt.optimize(black_box(sphere));
                    black_box(result.fitness)
                })
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_cmaes_sphere);
