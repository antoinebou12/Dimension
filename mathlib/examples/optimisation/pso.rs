//! Example: minimize the sphere function with PSO (particle swarm optimization).
//! Run with: `cargo run --example pso`
//! With logging: `RUST_LOG=mathlib=debug cargo run --example pso`

use mathlib::{PsoOptions, pso};

fn sphere_cost(x: &[f64]) -> f64 {
    x.iter().map(|v| v * v).sum()
}

fn main() {
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

    println!("PSO on sphere (dim = {}):", dim);
    println!("  best_cost: {}", result.best_cost);
    println!("  best_position: {:?}", result.best_position);
    println!("  iterations: {}", result.iterations);
}
