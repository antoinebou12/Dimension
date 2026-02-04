use mathlib::{dwt_haar_forward, dwt_haar_inverse};

#[test]
fn haar_roundtrip() {
    let x = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
    let fwd = dwt_haar_forward(&x);
    let inv = dwt_haar_inverse(&fwd);
    for (i, (&a, &b)) in x.iter().zip(inv.iter()).enumerate() {
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
fn haar_power_of_two() {
    for len in [2, 4, 8, 16, 32, 64] {
        let x: Vec<f64> = (0..len).map(|i| i as f64 * 0.5).collect();
        let fwd = dwt_haar_forward(&x);
        let inv = dwt_haar_inverse(&fwd);
        for (a, &b) in x.iter().zip(inv.iter()) {
            assert!((a - b).abs() < 1e-10);
        }
    }
}

#[test]
#[should_panic(expected = "even-length")]
fn haar_odd_length_panic() {
    let x = vec![1.0, 2.0, 3.0];
    let _ = dwt_haar_forward(&x);
}
