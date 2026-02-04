//! Integration tests for wasm vector bindings (demo: Vector add).

#![cfg(feature = "wasm")]

use mathlib::wasm::WasmVector;

#[test]
fn wasm_demo_vector_add() {
    // Demo: a = [1, 2, 3], b = [4, 5, 6], a + b = [5, 7, 9]
    let a = WasmVector::from_array(&[1.0, 2.0, 3.0]);
    let b = WasmVector::from_array(&[4.0, 5.0, 6.0]);
    let c = a.add(&b).unwrap();
    let arr = c.to_array();
    assert_eq!(arr.len(), 3);
    assert!((arr[0] - 5.0).abs() < 1e-10);
    assert!((arr[1] - 7.0).abs() < 1e-10);
    assert!((arr[2] - 9.0).abs() < 1e-10);
}
