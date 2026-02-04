//! Example: minimize the sphere function with CMA-ES.
//! Run with: `cargo run -p mathlib -F genetic --example cmaes`
//! With logging: `RUST_LOG=info,mathlib=debug cargo run -p mathlib -F genetic --example cmaes`

use mathlib::{CmaEsBuilder, CmaEsResult};
use tracing_subscriber::{EnvFilter, fmt};

fn sphere(x: &[f64]) -> f64 {
    x.iter().map(|&v| v * v).sum()
}

fn main() {
    fmt().with_env_filter(EnvFilter::from_default_env()).init();

    let dim = 6;
    let mean = vec![1.0; dim];
    let mut opt = CmaEsBuilder::new(dim, mean, 0.3)
        .max_generations(150)
        .seed(42)
        .build();
    let result: CmaEsResult = opt.optimize(sphere);
    println!("Best fitness: {}", result.fitness);
    println!("Solution: {:?}", result.solution);
    println!("Generations: {}", result.generations);
}
