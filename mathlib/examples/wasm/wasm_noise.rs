//! Example: WASM noise API (wave2d, perlin2d, fbm2dPerlin).
//! Run with: cargo run --example wasm_noise --features wasm
//!
//! Demonstrates wave, Perlin, and FBM noise using the same API that JavaScript
//! would use after `wasm-pack build --target web --features wasm`.

#![cfg_attr(not(feature = "wasm"), allow(dead_code))]

#[cfg(not(feature = "wasm"))]
fn main() {
    eprintln!("Build with: cargo run --example wasm_noise --features wasm");
}

#[cfg(feature = "wasm")]
fn main() {
    use mathlib::wasm::{fbm2d_perlin, perlin2d, wave2d, wave2d_params};

    println!("Wave 2D at (0.5, 0.5): {}", wave2d(0.5, 0.5));
    println!(
        "Wave 2D params (0, 0, 4π, 6π): {}",
        wave2d_params(
            0.0,
            0.0,
            4.0 * std::f64::consts::PI,
            6.0 * std::f64::consts::PI
        )
    );
    println!("Perlin 2D at (1.0, 2.0): {}", perlin2d(1.0, 2.0));
    println!(
        "FBM Perlin at (1, 1), 4 octaves: {}",
        fbm2d_perlin(1.0, 1.0, 4, 2.0, 0.5)
    );

    println!("\n5×5 Perlin grid (x,y in [0,2]):");
    for i in 0..5 {
        let y = 0.5 * i as f64;
        let row: Vec<String> = (0..5)
            .map(|j| format!("{:6.3}", perlin2d(0.5 * j as f64, y)))
            .collect();
        println!("  {}", row.join(" "));
    }
}
