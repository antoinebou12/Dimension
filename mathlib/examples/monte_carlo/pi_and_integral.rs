//! Monte Carlo example: π estimation and 1D integration.
//!
//! Run: `cargo run --example monte_carlo_pi_integral`

use mathlib::{estimate_pi, integrate_1d};
use std::f64::consts::PI;

fn main() {
    let seed = 42;
    let n = 200_000;

    let pi_est = estimate_pi(seed, n);
    println!("Monte Carlo π estimation (seed={}, n={}):", seed, n);
    println!(
        "  estimate = {:.6}, π = {:.6}, error = {:.6}",
        pi_est,
        PI,
        (pi_est - PI).abs()
    );

    // ∫₀¹ x² dx = 1/3
    let integral_x2 = integrate_1d(|x| x * x, 0.0, 1.0, n, seed + 1);
    println!("\nMonte Carlo ∫₀¹ x² dx (seed={}, n={}):", seed + 1, n);
    println!(
        "  estimate = {:.6}, 1/3 = {:.6}, error = {:.6}",
        integral_x2,
        1.0 / 3.0,
        (integral_x2 - 1.0 / 3.0).abs()
    );

    // ∫₀¹ 1 dx = 1
    let integral_one = integrate_1d(|_| 1.0, 0.0, 1.0, n, seed + 2);
    println!("\nMonte Carlo ∫₀¹ 1 dx (seed={}, n={}):", seed + 2, n);
    println!(
        "  estimate = {:.6}, 1 = 1.000000, error = {:.6}",
        integral_one,
        (integral_one - 1.0).abs()
    );
}
