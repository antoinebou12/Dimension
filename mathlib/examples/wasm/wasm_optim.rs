//! Example: WASM PSO API (psoMinimize, WasmPsoResult).
//! Run with: cargo run --example wasm_optim --features wasm
//!
//! Note: `pso_minimize` takes a JavaScript cost function callback, so it is only
//! usable from JavaScript. This example documents the API; the browser demo
//! (mathlib/wasm-demo) shows a full PSO minimization (e.g. sphere function).

#![cfg_attr(not(feature = "wasm"), allow(dead_code))]

#[cfg(not(feature = "wasm"))]
fn main() {
    eprintln!("Build with: cargo run --example wasm_optim --features wasm");
}

#[cfg(feature = "wasm")]
fn main() {
    println!("PSO (pso_minimize) is called from JavaScript with a cost function.");
    println!("See mathlib/wasm-demo for a browser demo that minimizes the sphere function.");
    println!(
        "Rust API: pso_minimize(lower, upper, num_particles, max_iters, cost_fn) -> WasmPsoResult"
    );
}
