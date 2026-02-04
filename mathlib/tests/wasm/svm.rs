//! Integration tests for wasm SVM bindings (demo: Linear SVM 2D).

#![cfg(feature = "wasm")]

use mathlib::wasm::{WasmMatrix, WasmSvm};

#[test]
fn wasm_demo_svm() {
    // Demo: 8 points in 2D, labels ±1; decision boundary w·x + b = 0; predictions [1,1,1,1,-1,-1,-1,-1]
    let data = vec![
        1.0, 2.0, 1.0, 2.0, 3.0, 4.0, 3.0, 4.0, 6.0, 7.0, 6.0, 7.0, 8.0, 9.0, 8.0, 9.0,
    ];
    let x = WasmMatrix::from_array(8, 2, &data).unwrap();
    let labels = [1.0, 1.0, 1.0, 1.0, -1.0, -1.0, -1.0, -1.0];
    let result = WasmSvm::train(&x, &labels).unwrap();
    let preds = result.predict_all(&x);
    assert_eq!(preds.len(), 8);
    for i in 0..4 {
        assert_eq!(preds[i], 1.0, "point {} predicted as +1", i);
    }
    for i in 4..8 {
        assert_eq!(preds[i], -1.0, "point {} predicted as -1", i);
    }
    let w = result.get_weights().to_array();
    let bias = result.get_bias();
    assert_eq!(w.len(), 2);
    assert!(bias.abs() > 0.1);
}
