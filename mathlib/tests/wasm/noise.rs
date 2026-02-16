//! Integration tests for wasm noise bindings (wave2d, perlin2d, fbm2dPerlin).
//! Run with: cargo test --features wasm wasm_noise

#![cfg(feature = "wasm")]

use mathlib::wasm::{fbm2d_perlin, perlin2d, wave2d, wave2d_params};

#[test]
fn wasm_wave2d() {
    let v = wave2d(0.5, 0.5);
    assert!((0.0..=1.0).contains(&v));
    let a = wave2d(0.3, 0.7);
    let b = wave2d(0.3, 0.7);
    assert!((a - b).abs() < 1e-10);
}

#[test]
fn wasm_wave2d_params() {
    let v = wave2d_params(
        0.0,
        0.0,
        4.0 * std::f64::consts::PI,
        6.0 * std::f64::consts::PI,
    );
    assert!((0.0..=1.0).contains(&v));
}

#[test]
fn wasm_perlin2d() {
    let v = perlin2d(1.0, 2.0);
    assert!((-1.5..=1.5).contains(&v));
    let a = perlin2d(0.5, 0.5);
    let b = perlin2d(0.5, 0.5);
    assert!((a - b).abs() < 1e-10);
}

#[test]
fn wasm_fbm2d_perlin() {
    let v = fbm2d_perlin(1.0, 1.0, 4, 2.0, 0.5);
    assert!(v.abs() < 2.0);
}
