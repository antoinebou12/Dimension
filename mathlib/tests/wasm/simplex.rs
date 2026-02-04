//! Integration tests for wasm simplex LP bindings (demo: Simplex LP).

#![cfg(feature = "wasm")]

use mathlib::wasm::{WasmMatrix, WasmSimplexResult, WasmVector};

#[test]
fn wasm_demo_simplex() {
    // Demo: min c'x, Ax = b, x >= 0; c = [1,1], A = [[1,2],[1,0]], b = [4,2] -> optimal, objective 3, x = [2, 1]
    let c = WasmVector::from_array(&[1.0, 1.0]);
    let a = WasmMatrix::from_array(2, 2, &[1.0, 1.0, 2.0, 0.0]).unwrap();
    let b = WasmVector::from_array(&[4.0, 2.0]);
    let result = WasmSimplexResult::new(&c, &a, &b).unwrap();
    assert_eq!(result.get_status(), "optimal");
    assert!((result.get_objective() - 3.0).abs() < 1e-10);
    let x = result.get_x().to_array();
    assert!((x[0] - 2.0).abs() < 1e-10);
    assert!((x[1] - 1.0).abs() < 1e-10);
}
