use mathlib::{dct2_forward, dct2_inverse};

#[test]
fn dct_roundtrip() {
    let x = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
    let fwd = dct2_forward(&x).unwrap();
    let inv = dct2_inverse(&fwd).unwrap();
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
fn dct_constant_signal() {
    let n = 16;
    let x = vec![3.0; n];
    let fwd = dct2_forward(&x).unwrap();
    assert!((fwd[0] - 3.0 * (n as f64).sqrt()).abs() < 1e-10);
    for v in fwd.iter().skip(1) {
        assert!(v.abs() < 1e-10, "DCT of constant should be zero for k>0");
    }
}

#[test]
fn dct_empty_input() {
    let x: Vec<f64> = vec![];
    let err = dct2_forward(&x).unwrap_err();
    assert_eq!(err, mathlib::TransformsError::EmptyInput);
}
