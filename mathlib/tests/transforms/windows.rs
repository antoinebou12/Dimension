use mathlib::{apply_window, apply_window_in_place, blackman, hamming, hann, tukey};

#[test]
fn hann_symmetry() {
    let w = hann(64);
    assert_eq!(w.len(), 64);
    assert!((w[0] - 0.0).abs() < 1e-10);
    assert!((w[63] - 0.0).abs() < 1e-10);
    assert!(w[32] > 0.99 && w[32] <= 1.0, "Hann center should be near 1");
}

#[test]
fn hann_n1() {
    let w = hann(1);
    assert_eq!(w.len(), 1);
    assert!((w[0] - 1.0).abs() < 1e-10);
}

#[test]
fn apply_window_roundtrip() {
    let signal = vec![1.0, 2.0, 3.0, 4.0];
    let window = hann(4);
    let mut out = vec![0.0; 4];
    apply_window(&signal, &window, &mut out);
    for (s, o) in signal.iter().zip(out.iter()) {
        assert!(
            o.abs() <= s.abs() + 1e-10,
            "windowed value should be bounded"
        );
    }
}

#[test]
fn test_apply_window_in_place() {
    let mut signal = vec![1.0, 2.0, 3.0, 4.0];
    let window = hann(4);
    apply_window_in_place(&mut signal, &window);
    assert!(signal[0].abs() < 1e-10);
    assert!(signal[3].abs() < 1e-10);
}

#[test]
fn blackman_sum() {
    let w = blackman(32);
    let sum: f64 = w.iter().sum();
    assert!(
        sum > 0.0 && sum < 32.0,
        "Blackman window sum should be in (0, N)"
    );
}

#[test]
fn tukey_alpha_zero_rectangular() {
    let w = tukey(32, 0.0);
    assert_eq!(w.len(), 32);
    for &v in &w {
        assert!((v - 1.0).abs() < 1e-10);
    }
}

#[test]
fn tukey_alpha_one_hann_like() {
    let w = tukey(64, 1.0);
    let h = hann(64);
    for (a, b) in w.iter().zip(h.iter()) {
        assert!((a - b).abs() < 1e-10);
    }
}

#[test]
fn tukey_alpha_half() {
    let w = tukey(32, 0.5);
    assert_eq!(w.len(), 32);
    assert!(w[0] >= 0.0 && w[0] < 0.1, "Tukey taper starts near 0");
    assert!(w[15] > 0.99 && w[15] <= 1.0);
    assert!(w[31] >= 0.0 && w[31] < 0.1, "Tukey taper ends near 0");
}

#[test]
fn hamming_values() {
    let w = hamming(16);
    assert_eq!(w.len(), 16);
    assert!((w[0] - 0.08).abs() < 0.02, "Hamming w[0] should be ~0.08");
    assert!(w[8] > 0.9 && w[8] <= 1.0, "Hamming center should be near 1");
}
