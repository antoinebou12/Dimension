//! Example: WASM transform API (FFT, DCT, wavelets, convolution, windows).
//! Run with: cargo run --example wasm_transforms --features wasm
//!
//! Demonstrates the same API that JavaScript would use after
//! `wasm-pack build --target web --features wasm`.

#[cfg(not(feature = "wasm"))]
fn main() {
    eprintln!("Build with: cargo run --example wasm_transforms --features wasm");
}

#[cfg(feature = "wasm")]
fn main() {
    use mathlib::wasm::{
        apply_window_wasm, dct2_forward_wasm, dct2_inverse_wasm, dwt_haar_forward_wasm,
        dwt_haar_inverse_wasm, fft_forward_real_wasm, fft_inverse_wasm, hann_wasm,
    };

    let n = 64_usize;
    let signal: Vec<f64> = (0..n)
        .map(|i| (2.0 * std::f64::consts::PI * 5.0 * i as f64 / n as f64).sin())
        .collect();

    println!("WASM Transforms example (signal length = {})", n);

    let spectrum = fft_forward_real_wasm(&signal).unwrap();
    let inv = fft_inverse_wasm(&spectrum).unwrap();
    let max_err = signal
        .iter()
        .zip(inv.iter())
        .map(|(a, b)| (a - b).abs())
        .fold(0.0_f64, f64::max);
    println!("FFT round-trip max error: {:.2e}", max_err);

    let dct_coeffs = dct2_forward_wasm(&signal).unwrap();
    let dct_restored = dct2_inverse_wasm(&dct_coeffs).unwrap();
    let dct_err = signal
        .iter()
        .zip(dct_restored.iter())
        .map(|(a, b)| (a - b).abs())
        .fold(0.0_f64, f64::max);
    println!("DCT round-trip max error: {:.2e}", dct_err);

    let haar_coeffs = dwt_haar_forward_wasm(&signal);
    let haar_restored = dwt_haar_inverse_wasm(&haar_coeffs);
    let haar_err = signal
        .iter()
        .zip(haar_restored.iter())
        .map(|(a, b)| (a - b).abs())
        .fold(0.0_f64, f64::max);
    println!("Haar DWT round-trip max error: {:.2e}", haar_err);

    let window = hann_wasm(n);
    let windowed = apply_window_wasm(&signal, &window).unwrap();
    println!("Hann window applied (first sample): {:.4}", windowed[0]);
}
