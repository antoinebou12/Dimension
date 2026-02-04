//! Integration tests for PSO.

use mathlib::{PsoOptions, pso};

fn sphere_cost(x: &[f64]) -> f64 {
    x.iter().map(|v| v * v).sum()
}

#[test]
fn pso_sphere_converges() {
    let dim = 4usize;
    let low = vec![-5.0; dim];
    let high = vec![5.0; dim];
    let result = pso(
        (low, high),
        20,
        sphere_cost,
        100,
        Some(PsoOptions::default()),
    );
    assert!(
        result.best_cost < 1.0,
        "sphere cost should be small, got {}",
        result.best_cost
    );
    for &x in &result.best_position {
        assert!(x.abs() < 2.0, "best position should be near 0");
    }
    assert_eq!(result.iterations, 100);
}

#[test]
fn pso_bounds_respected() {
    let low = vec![-2.0, -3.0];
    let high = vec![2.0, 3.0];
    let result = pso((low.clone(), high.clone()), 10, sphere_cost, 5, None);
    for (i, &x) in result.best_position.iter().enumerate() {
        assert!(
            x >= low[i],
            "position[{}] {} below lower bound {}",
            i,
            x,
            low[i]
        );
        assert!(
            x <= high[i],
            "position[{}] {} above upper bound {}",
            i,
            x,
            high[i]
        );
    }
}

#[test]
fn pso_deterministic() {
    let low = vec![-1.0, -1.0];
    let high = vec![1.0, 1.0];
    let r1 = pso((low.clone(), high.clone()), 8, sphere_cost, 20, None);
    let r2 = pso((low, high), 8, sphere_cost, 20, None);
    assert_eq!(r1.best_cost.to_bits(), r2.best_cost.to_bits());
    assert_eq!(r1.best_position.len(), r2.best_position.len());
    for (a, b) in r1.best_position.iter().zip(r2.best_position.iter()) {
        assert_eq!(a.to_bits(), b.to_bits());
    }
}

#[test]
fn pso_population_size() {
    let low = vec![0.0, 0.0];
    let high = vec![1.0, 1.0];
    let result = pso((low, high), 1, sphere_cost, 3, None);
    assert_eq!(result.best_position.len(), 2);
    let result = pso((vec![0.0, 0.0], vec![1.0, 1.0]), 40, sphere_cost, 2, None);
    assert_eq!(result.best_position.len(), 2);
}
