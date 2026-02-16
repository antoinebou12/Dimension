//! Example: minimize the sphere and Rosenbrock functions with L-BFGS-B (box constraints).
//! Run with: `cargo run --example lbfgsb`
//! With logging: `RUST_LOG=mathlib=debug cargo run --example lbfgsb`

use mathlib::{LbfgsbOptions, lbfgsb};

fn main() {
    // 1. Sphere: min x² + y², unconstrained (large box)
    let x0 = vec![5.0_f64, 5.0];
    let lower = vec![-1e30_f64, -1e30];
    let upper = vec![1e30_f64, 1e30];
    let cost = |x: &[f64]| x[0] * x[0] + x[1] * x[1];
    let gradient = |x: &[f64], g: &mut [f64]| {
        g[0] = 2.0 * x[0];
        g[1] = 2.0 * x[1];
    };
    let opts = LbfgsbOptions {
        max_iters: 500,
        tol: 1e-10,
        m: 5,
        ..Default::default()
    };
    let result = lbfgsb(&x0, &lower, &upper, cost, gradient, &opts);
    println!("L-BFGS-B on sphere (x0 = [5, 5]), unconstrained:");
    println!("  x: {:?}", result.x);
    println!("  cost: {}", result.cost);
    println!("  iterations: {}", result.iterations);

    // 2. Rosenbrock in box [-2, 2]²
    let rosenbrock = |x: &[f64]| (1.0 - x[0]).powi(2) + 100.0 * (x[1] - x[0] * x[0]).powi(2);
    let rosenbrock_grad = |x: &[f64], g: &mut [f64]| {
        g[0] = -2.0 * (1.0 - x[0]) - 400.0 * x[0] * (x[1] - x[0] * x[0]);
        g[1] = 200.0 * (x[1] - x[0] * x[0]);
    };
    let x0_r = vec![0.0_f64, 0.0];
    let lower_r = vec![-2.0_f64, -2.0];
    let upper_r = vec![2.0_f64, 2.0];
    let result_r = lbfgsb(
        &x0_r,
        &lower_r,
        &upper_r,
        rosenbrock,
        rosenbrock_grad,
        &opts,
    );
    println!("\nL-BFGS-B on Rosenbrock in [-2,2]²:");
    println!("  x: {:?}", result_r.x);
    println!("  cost: {}", result_r.cost);
    println!("  iterations: {}", result_r.iterations);
}
