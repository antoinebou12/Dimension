//! Integration tests for wasm matrix bindings (demos: Matrix multiply, SVD, WasmMatrix32).

#![cfg(feature = "wasm")]

use mathlib::wasm::{WasmMatrix, WasmMatrix32, WasmSvd};

#[test]
fn wasm_demo_matrix_mul() {
    // Demo: A = identity 3×3, B = 3×3, C = A×B (column-major)
    let a = WasmMatrix::from_array(3, 3, &[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]).unwrap();
    let b = WasmMatrix::from_array(3, 3, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0]).unwrap();
    let c = a.mul(&b).unwrap();
    let out = c.to_array();
    assert_eq!(c.rows(), 3);
    assert_eq!(c.cols(), 3);
    assert!((out[0] - 1.0).abs() < 1e-10 && (out[1] - 2.0).abs() < 1e-10);
    assert!((out[3] - 4.0).abs() < 1e-10 && (out[4] - 5.0).abs() < 1e-10);
    assert!((out[6] - 7.0).abs() < 1e-10 && (out[7] - 8.0).abs() < 1e-10);
}

#[test]
fn wasm_demo_svd() {
    // Demo: 3×2 matrix, economical SVD; check U, V, sigma dimensions and sigma values
    let data = vec![1.0, 0.0, 0.0, 1.0, 1.0, 1.0];
    let m = WasmMatrix::from_array(3, 2, &data).unwrap();
    let svd: WasmSvd = m.svd_econ();
    let sigma = svd.get_sigma().to_array();
    let u = svd.get_u();
    let v = svd.get_v();
    assert_eq!(u.rows(), 3);
    assert_eq!(u.cols(), 2);
    assert_eq!(v.rows(), 2);
    assert_eq!(v.cols(), 2);
    assert_eq!(sigma.len(), 2);
    assert!(sigma[0] >= sigma[1] && sigma[0] > 0.0);
}

#[test]
fn wasm_matrix32_rotation_and_transform() {
    let rot = WasmMatrix32::rotation(0.0, 0.0, std::f32::consts::FRAC_PI_2);
    let pt = rot.transform_point(1.0, 0.0, 0.0).unwrap();
    assert_eq!(pt.len(), 3);
    let norm_sq = pt[0] * pt[0] + pt[1] * pt[1] + pt[2] * pt[2];
    assert!((norm_sq - 1.0).abs() < 1e-5, "rotation preserves length");
}
