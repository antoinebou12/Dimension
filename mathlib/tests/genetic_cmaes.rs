//! Tests for CMA-ES. Run with `cargo test -p mathlib -F genetic`.

#![cfg(feature = "genetic")]

use mathlib::{CmaEs, CmaEsBuilder};

/// Sphere: minimize Σ xᵢ²; optimum 0 at origin.
fn sphere(x: &[f64]) -> f64 {
    x.iter().map(|&v| v * v).sum()
}

/// Rosenbrock: f(x) = Σ 100(x_{i+1} - x_i²)² + (1 - x_i)²; optimum 0 at (1,1,...,1).
fn rosenbrock(x: &[f64]) -> f64 {
    let n = x.len();
    assert!(n >= 2);
    (0..n - 1)
        .map(|i| 100.0 * (x[i + 1] - x[i] * x[i]).powi(2) + (1.0 - x[i]).powi(2))
        .sum()
}

#[test]
fn cmaes_sphere_converges() {
    let dim = 4;
    let mean = vec![2.0; dim];
    let mut opt = CmaEsBuilder::new(dim, mean, 0.3)
        .max_generations(200)
        .seed(123)
        .build();
    let result = opt.optimize(sphere);
    assert!(
        result.fitness < 1e-6,
        "sphere should converge near 0, got {}",
        result.fitness
    );
    for &v in &result.solution {
        assert!(
            v.abs() < 0.1,
            "solution component should be near 0, got {}",
            v
        );
    }
}

#[test]
fn cmaes_rosenbrock() {
    let dim = 4;
    let mean = vec![0.5; dim];
    let mut opt = CmaEsBuilder::new(dim, mean, 0.3)
        .max_generations(400)
        .seed(456)
        .build();
    let result = opt.optimize(rosenbrock);
    assert!(
        result.fitness < 10.0,
        "Rosenbrock should reach reasonable solution, got fitness {}",
        result.fitness
    );
}
