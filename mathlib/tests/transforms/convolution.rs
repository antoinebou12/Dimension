use mathlib::{Matrix, Storage, conv_1d, conv_1d_same, conv_2d};

#[test]
fn conv_1d_identity_kernel() {
    let signal = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let kernel = vec![1.0];
    let out = conv_1d(&signal, &kernel);
    assert_eq!(out.len(), 5);
    for (a, &b) in signal.iter().zip(out.iter()) {
        assert!((a - b).abs() < 1e-10);
    }
}

#[test]
fn conv_1d_known() {
    let signal = vec![1.0, 2.0, 3.0];
    let kernel = vec![1.0, 1.0];
    let out = conv_1d(&signal, &kernel);
    assert_eq!(out.len(), 4);
    assert!((out[0] - 1.0).abs() < 1e-10);
    assert!((out[1] - 3.0).abs() < 1e-10);
    assert!((out[2] - 5.0).abs() < 1e-10);
    assert!((out[3] - 3.0).abs() < 1e-10);
}

#[test]
fn conv_1d_same_length() {
    let signal = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let kernel = vec![1.0, 0.0, 1.0];
    let out = conv_1d_same(&signal, &kernel);
    assert_eq!(out.len(), 5);
}

#[test]
fn conv_2d_identity() {
    let mut m = Matrix::with_storage(3, 3, Storage::Column);
    for i in 0..3 {
        for j in 0..3 {
            m.set(i, j, (i * 3 + j) as f64);
        }
    }
    let mut k = Matrix::with_storage(3, 3, Storage::Column);
    k.set_zero();
    k.set(1, 1, 1.0);
    let out = conv_2d(&m, &k);
    assert_eq!(out.rows(), 3);
    assert_eq!(out.cols(), 3);
    assert!((out.get(1, 1) - 4.0).abs() < 1e-10);
}
