use mathlib::{Complex64, TransformsError, fft_forward, fft_forward_real, fft_inverse};

#[test]
fn fft_roundtrip_complex() {
    let n = 256;
    let x: Vec<Complex64> = (0..n)
        .map(|i| {
            let t = 2.0 * std::f64::consts::PI * 10.0 * i as f64 / n as f64;
            Complex64::new(t.cos(), t.sin())
        })
        .collect();
    let fwd = fft_forward(&x).unwrap();
    let inv = fft_inverse(&fwd).unwrap();
    for (i, (a, b)) in x.iter().zip(inv.iter()).enumerate() {
        assert!(
            (a.re - b.re).abs() < 1e-10 && (a.im - b.im).abs() < 1e-10,
            "at {}: original {:?} vs reconstructed {:?}",
            i,
            a,
            b
        );
    }
}

#[test]
fn fft_roundtrip_real() {
    let n = 256;
    let x: Vec<f64> = (0..n)
        .map(|i| (2.0 * std::f64::consts::PI * 5.0 * i as f64 / n as f64).sin())
        .collect();
    let fwd = fft_forward_real(&x).unwrap();
    let inv_complex = fft_inverse(&fwd).unwrap();
    let inv_real: Vec<f64> = inv_complex.iter().map(|c| c.re).collect();
    for (i, (a, &b)) in x.iter().zip(inv_real.iter()).enumerate() {
        assert!(
            (a - b).abs() < 1e-10,
            "at {}: original {} vs reconstructed {}",
            i,
            a,
            b
        );
    }
}

#[test]
fn fft_dc_component() {
    let n = 128;
    let x: Vec<f64> = vec![1.0; n];
    let fwd = fft_forward_real(&x).unwrap();
    assert!(
        (fwd[0].re - n as f64).abs() < 1e-10 && fwd[0].im.abs() < 1e-10,
        "DC component of constant signal should be N"
    );
}

#[test]
fn fft_invalid_length() {
    let x = vec![Complex64::new(1.0, 0.0); 100];
    let err = fft_forward(&x).unwrap_err();
    assert_eq!(err, TransformsError::LengthNotPowerOfTwo(100));
}

#[test]
fn fft_empty_input() {
    let x: Vec<Complex64> = vec![];
    let err = fft_forward(&x).unwrap_err();
    assert_eq!(err, TransformsError::EmptyInput);
}
