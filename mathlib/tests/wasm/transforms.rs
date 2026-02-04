//! Integration tests for wasm transforms bindings.

use mathlib::wasm::{
    apply_window_wasm, blackman_wasm, conv_1d_same_wasm, conv_1d_wasm, dct2_forward_wasm,
    dct2_inverse_wasm, dwt_haar_forward_wasm, dwt_haar_inverse_wasm, fft_forward_real_wasm,
    fft_inverse_wasm, hamming_wasm, hann_wasm, tukey_wasm,
};

#[test]
fn wasm_hann() {
    let w = hann_wasm(64);
    assert_eq!(w.len(), 64);
    assert!(w[0].abs() < 1e-10);
    assert!(w[32] > 0.99);
}

#[test]
fn wasm_tukey() {
    let w = tukey_wasm(32, 0.5);
    assert_eq!(w.len(), 32);
    let w_rect = tukey_wasm(32, 0.0);
    assert!(w_rect.iter().all(|&v| (v - 1.0).abs() < 1e-10));
}

#[test]
fn wasm_fft_roundtrip() {
    let signal: Vec<f64> = (0..256)
        .map(|i| (2.0 * std::f64::consts::PI * 5.0 * i as f64 / 256.0).sin())
        .collect();
    let spectrum = fft_forward_real_wasm(&signal).unwrap();
    assert_eq!(spectrum.len(), 512);
    let inv = fft_inverse_wasm(&spectrum).unwrap();
    assert_eq!(inv.len(), 256);
    for (a, &b) in signal.iter().zip(inv.iter()) {
        assert!((a - b).abs() < 1e-8);
    }
}

#[test]
fn wasm_dct_roundtrip() {
    let signal = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
    let coeffs = dct2_forward_wasm(&signal).unwrap();
    let restored = dct2_inverse_wasm(&coeffs).unwrap();
    for (a, b) in signal.iter().zip(restored.iter()) {
        assert!((a - b).abs() < 1e-10);
    }
}

#[test]
fn wasm_haar_roundtrip() {
    let signal = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
    let coeffs = dwt_haar_forward_wasm(&signal);
    let restored = dwt_haar_inverse_wasm(&coeffs);
    for (a, b) in signal.iter().zip(restored.iter()) {
        assert!((a - b).abs() < 1e-10);
    }
}

#[test]
fn wasm_conv1d() {
    let signal = vec![1.0, 2.0, 3.0];
    let kernel = vec![1.0, 1.0];
    let out = conv_1d_wasm(&signal, &kernel);
    assert_eq!(out.len(), 4);
    assert!((out[0] - 1.0).abs() < 1e-10);
    assert!((out[1] - 3.0).abs() < 1e-10);
}

#[test]
fn wasm_apply_window() {
    let signal = vec![1.0, 2.0, 3.0, 4.0];
    let window = hann_wasm(4);
    let windowed = apply_window_wasm(&signal, &window).unwrap();
    assert_eq!(windowed.len(), 4);
}
