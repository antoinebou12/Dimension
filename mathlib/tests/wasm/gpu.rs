//! Integration tests for wasm GPU bindings (initGpuAsync, matmulF32GpuAsync, dotF32GpuAsync, etc.).
//! Run with: cargo test --features "wasm gpu" wasm

#![cfg(all(feature = "wasm", feature = "gpu"))]

use mathlib::gpu;
use mathlib::wasm::{WasmMatrix32, WasmVector};

/// Tests that GPU init and availability can be queried.
#[test]
fn wasm_gpu_availability_query() {
    // Before init, gpuAvailable may be false; after init_async it may be true.
    let _ = gpu::is_available();
}

/// Tests that dotF32 and norm work (CPU path when GPU not init'd; tests the API).
#[test]
fn wasm_gpu_dot_norm_api() {
    let a = WasmVector::from_array(&[1.0_f64, 2.0, 3.0]);
    let b = WasmVector::from_array(&[4.0_f64, 5.0, 6.0]);
    let dot = a.dot(&b).unwrap();
    assert!((dot - 32.0).abs() < 1e-10);
    let norm_a = a.norm();
    assert!((norm_a - 14.0_f64.sqrt()).abs() < 1e-10);
}

/// Tests WasmMatrix32 matmul (uses CPU when GPU not init'd; API smoke test).
#[test]
fn wasm_gpu_matrix32_matmul_api() {
    let a = WasmMatrix32::from_array(2, 2, &[1.0, 2.0, 3.0, 4.0]).unwrap();
    let b = WasmMatrix32::from_array(2, 2, &[1.0, 0.0, 0.0, 1.0]).unwrap();
    let c = a.mul(&b).unwrap();
    let out = c.to_array();
    assert_eq!(out.len(), 4);
    assert!((out[0] - 1.0).abs() < 1e-5);
    assert!((out[1] - 2.0).abs() < 1e-5);
}
