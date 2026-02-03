//! Example: minimize the sphere function with gradient descent (uses line search internally).
//! Run with: `cargo run --example argmin`
//! With logging: `RUST_LOG=mathlib=debug cargo run --example argmin`

use mathlib::{GradientDescentOptions, gradient_descent};

fn main() {
    let x0 = vec![5.0_f64, 5.0];
    let cost = |x: &[f64]| x[0] * x[0] + x[1] * x[1];
    let gradient = |x: &[f64], g: &mut [f64]| {
        g[0] = 2.0 * x[0];
        g[1] = 2.0 * x[1];
    };
    let opts = GradientDescentOptions {
        max_iters: 500,
        tol: 1e-9,
        ..Default::default()
    };

    let result = gradient_descent(&x0, cost, gradient, &opts);

    println!("Gradient descent on 2D sphere (x0 = [5, 5]):");
    println!("  x: {:?}", result.x);
    println!("  cost: {}", result.cost);
    println!("  iterations: {}", result.iterations);
}
