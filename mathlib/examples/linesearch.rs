//! Example: one step of backtracking line search on the 2D sphere cost.
//! Run with: `cargo run --example linesearch`
//! With logging: `RUST_LOG=mathlib=debug cargo run --example linesearch`

use mathlib::{LineSearchOptions, armijo, backtracking};

fn main() {
    let x = [1.0_f64, 0.0];
    let d = [-1.0_f64, 0.0];
    let f = 1.0;
    let g_dot_d = -2.0;
    let cost = |x: &[f64]| x[0] * x[0] + x[1] * x[1];
    let opts = LineSearchOptions::default();
    let mut scratch = [0.0_f64; 2];

    let alpha = backtracking(&x, &d, f, g_dot_d, cost, &opts, &mut scratch);

    println!("Line search (backtracking) on 2D sphere:");
    println!("  x = {:?}, d = {:?}", x, d);
    println!("  alpha = {}", alpha);
    println!("  x + alpha*d = {:?}", scratch);
    println!("  cost at new point: {}", cost(&scratch));

    let mut scratch2 = [0.0_f64; 2];
    let alpha_armijo = armijo(&x, &d, f, g_dot_d, cost, &opts, &mut scratch2);
    println!("  armijo alpha = {}", alpha_armijo);
}
