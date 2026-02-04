//! Example: FFT, DCT, wavelets, convolution, and spectral windows.
//!
//! Run with: `cargo run -p mathlib --example transforms`

use mathlib::{
    apply_window, blackman, conv_1d, conv_1d_same, dct2_forward, dct2_inverse, dwt_haar_forward,
    dwt_haar_inverse, fft_forward_real, fft_inverse, hamming, hann, tukey,
};

fn main() {
    let n = 64_usize;
    let signal: Vec<f64> = (0..n)
        .map(|i| (2.0 * std::f64::consts::PI * 5.0 * i as f64 / n as f64).sin())
        .collect();

    println!("Transforms example (signal length = {})", n);

    // FFT
    let spectrum = fft_forward_real(&signal).unwrap();
    let inv = fft_inverse(&spectrum).unwrap();
    let inv_real: Vec<f64> = inv.iter().map(|c| c.re).collect();
    let max_err = signal
        .iter()
        .zip(inv_real.iter())
        .map(|(a, b)| (a - b).abs())
        .fold(0.0_f64, f64::max);
    println!("\nFFT round-trip max error: {:.2e}", max_err);
    println!("  DC component (spectrum[0].re): {:.4}", spectrum[0].re);

    // DCT
    let dct_coeffs = dct2_forward(&signal).unwrap();
    let dct_restored = dct2_inverse(&dct_coeffs).unwrap();
    let dct_err = signal
        .iter()
        .zip(dct_restored.iter())
        .map(|(a, b)| (a - b).abs())
        .fold(0.0_f64, f64::max);
    println!("\nDCT round-trip max error: {:.2e}", dct_err);

    // Haar wavelets
    let haar_coeffs = dwt_haar_forward(&signal);
    let haar_restored = dwt_haar_inverse(&haar_coeffs);
    let haar_err = signal
        .iter()
        .zip(haar_restored.iter())
        .map(|(a, b)| (a - b).abs())
        .fold(0.0_f64, f64::max);
    println!("\nHaar DWT round-trip max error: {:.2e}", haar_err);

    // Convolution
    let kernel = vec![0.25, 0.5, 0.25];
    let conv_full = conv_1d(&signal, &kernel);
    let conv_same = conv_1d_same(&signal, &kernel);
    println!(
        "\nConvolution: full len={}, same len={}",
        conv_full.len(),
        conv_same.len()
    );

    // Windows
    let w_hann = hann(n);
    let w_hamming = hamming(n);
    let _w_blackman = blackman(n);
    let w_tukey05 = tukey(n, 0.5);
    let w_tukey0 = tukey(n, 0.0);
    let w_tukey1 = tukey(n, 1.0);
    println!("\nWindows (first 5 values):");
    println!("  Hann:    {:?}", &w_hann[..5.min(n)]);
    println!("  Hamming: {:?}", &w_hamming[..5.min(n)]);
    println!("  Tukey(α=0.5): {:?}", &w_tukey05[..5.min(n)]);
    println!("  Tukey(α=0) rect: w[0]={:.2}", w_tukey0[0]);
    println!("  Tukey(α=1) Hann-like: w[0]={:.4}", w_tukey1[0]);

    let mut windowed = vec![0.0; n];
    apply_window(&signal, &w_hann, &mut windowed);
    let spectrum_windowed = fft_forward_real(&windowed).unwrap();
    println!("\nWindowed FFT DC (Hann): {:.4}", spectrum_windowed[0].re);
}
