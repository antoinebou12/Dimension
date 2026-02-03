use mathlib::Vector;

#[test]
fn vector_dynamic() {
    let mut v = Vector::with_capacity(5);
    v.set_zero();
    assert_eq!(v.rows(), 5);
    v.resize(3);
    assert_eq!(v.rows(), 3);
    v.set(0, 1.0);
    v.set(1, 2.0);
    v.set(2, 3.0);
    let n = v.norm();
    let expected = (1.0_f64 * 1.0 + 2.0 * 2.0 + 3.0 * 3.0).sqrt();
    assert!((n - expected).abs() < 1e-9);
}

#[test]
fn vector_dot_norm() {
    let mut u = Vector::with_capacity(3);
    u.set(0, 1.0);
    u.set(1, 0.0);
    u.set(2, 0.0);
    let mut v = Vector::with_capacity(3);
    v.set(0, 1.0);
    v.set(1, 2.0);
    v.set(2, 0.0);
    let dp = u.dot(&v);
    assert!((dp - 1.0_f64).abs() < 1e-9);
    assert!((u.norm() - 1.0_f64).abs() < 1e-9);
}
