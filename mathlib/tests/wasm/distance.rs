//! Integration tests for wasm distance bindings (demo: Distance metrics).

#![cfg(feature = "wasm")]

use mathlib::wasm::{WasmDistance, WasmVector};

#[test]
fn wasm_demo_distance() {
    // Demo: a = [1, 0, 0], b = [0.6, 0.8, 0]; euclidean, manhattan, cosine similarity/distance
    let a = WasmVector::from_array(&[1.0, 0.0, 0.0]);
    let b = WasmVector::from_array(&[0.6, 0.8, 0.0]);
    let eucl = a.euclidean_distance(&b).unwrap();
    let manh = WasmDistance::manhattan(&a, &b).unwrap();
    let cos_sim = WasmDistance::cosine_similarity(&a, &b).unwrap();
    let cos_dist = WasmDistance::cosine_distance(&a, &b).unwrap();
    assert!((eucl - 0.8944).abs() < 0.001);
    assert!((manh - 1.2).abs() < 1e-10);
    assert!((cos_sim - 0.6).abs() < 1e-10);
    assert!((cos_dist - 0.4).abs() < 1e-10);
}
