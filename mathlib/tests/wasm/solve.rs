//! Integration tests for wasm solve and damped least-squares bindings.
//! Run with: cargo test --features wasm wasm_solve

#![cfg(feature = "wasm")]

use mathlib::wasm::{WasmMatrix, WasmVector};

#[test]
fn wasm_demo_solve() {
    // Demo: 2×2 system [[2,1],[1,2]] x = [3, 3] -> x = [1, 1]
    let a = WasmMatrix::from_array(2, 2, &[2.0, 1.0, 1.0, 2.0]).unwrap();
    let b = WasmVector::from_array(&[3.0, 3.0]);
    let x = a.solve(&b).unwrap();
    let arr = x.to_array();
    assert_eq!(arr.len(), 2);
    assert!((arr[0] - 1.0).abs() < 1e-10);
    assert!((arr[1] - 1.0).abs() < 1e-10);
}

#[test]
fn wasm_demo_damped_least_squares() {
    // 3×2 overdetermined: use λ² > 0 so normal equations are non-singular.
    let a = WasmMatrix::from_array(3, 2, &[1.0, 0.0, 1.0, 0.0, 1.0, 1.0]).unwrap();
    let b = WasmVector::from_array(&[1.0, 2.0, 3.0]);
    let x_small = a.damped_least_squares(&b, 0.01).unwrap();
    let x_large = a.damped_least_squares(&b, 1.0).unwrap();
    let arr_small = x_small.to_array();
    let arr_large = x_large.to_array();
    assert_eq!(arr_small.len(), 2);
    assert_eq!(arr_large.len(), 2);
    let norm_small = (arr_small[0].powi(2) + arr_small[1].powi(2)).sqrt();
    let norm_large = (arr_large[0].powi(2) + arr_large[1].powi(2)).sqrt();
    assert!(
        norm_large <= norm_small + 1e-10,
        "larger damping should not increase solution norm"
    );
}
