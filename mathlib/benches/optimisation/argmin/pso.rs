//! Benchmarks for PSO (sphere cost).

use criterion::{BenchmarkId, Criterion, criterion_group};
use mathlib::{PsoOptions, pso};
use std::hint::black_box;

fn sphere_cost(x: &[f64]) -> f64 {
    x.iter().map(|v| v * v).sum()
}

fn synthetic_bounds(dim: usize, low: f64, high: f64) -> (Vec<f64>, Vec<f64>) {
    (vec![low; dim], vec![high; dim])
}

pub fn bench_pso(c: &mut Criterion) {
    let mut group = c.benchmark_group("pso");

    let configs = [(40, 8, 100), (80, 16, 200)];

    for (num_particles, dim, iters) in configs {
        let (low, high) = synthetic_bounds(dim, -5.0, 5.0);
        let bounds = (low, high);
        group.bench_with_input(
            BenchmarkId::new(
                "pso_sphere",
                format!("{}p_{}d_{}i", num_particles, dim, iters),
            ),
            &(bounds, num_particles, iters),
            |b, (bounds, num_particles, iters)| {
                b.iter(|| {
                    let result = pso(
                        black_box(bounds.clone()),
                        *num_particles,
                        black_box(sphere_cost),
                        *iters,
                        Some(PsoOptions::default()),
                    );
                    black_box(result)
                })
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_pso);
