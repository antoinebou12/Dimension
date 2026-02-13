use mathlib::{CpuExecutor, Executor, Vector};

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

#[test]
fn vector_new_and_default() {
    let v: Vector<f64> = Vector::new();
    assert_eq!(v.rows(), 0);
    let v2: Vector<f64> = Default::default();
    assert_eq!(v2.rows(), 0);
}

#[test]
fn vector_from_slice() {
    let data = [1.0_f64, 2.0, 3.0, 4.0];
    let v = Vector::from_slice(&data);
    assert_eq!(v.rows(), 4);
    assert!((v.get(0) - 1.0_f64).abs() < 1e-9);
    assert!((v.get(3) - 4.0_f64).abs() < 1e-9);
}

#[test]
fn vector_map() {
    let v = Vector::from_slice(&[1.0_f64, 2.0, 3.0]);
    let doubled = v.map(|x| x * 2.0_f64);
    assert!((doubled.get(0) - 2.0_f64).abs() < 1e-9);
    assert!((doubled.get(1) - 4.0_f64).abs() < 1e-9);
    assert!((doubled.get(2) - 6.0_f64).abs() < 1e-9);
}

#[test]
fn vector_zip_map() {
    let a = Vector::from_slice(&[1.0_f64, 2.0, 3.0]);
    let b = Vector::from_slice(&[4.0_f64, 5.0, 6.0]);
    let sum = a.zip_map(&b, |x, y| x + y);
    assert!((sum.get(0) - 5.0_f64).abs() < 1e-9);
    assert!((sum.get(1) - 7.0_f64).abs() < 1e-9);
    assert!((sum.get(2) - 9.0_f64).abs() < 1e-9);
}

#[test]
fn vector_normalize() {
    let v = Vector::from_slice(&[3.0_f64, 4.0]);
    let n = v.normalize();
    assert!((n.norm() - 1.0_f64).abs() < 1e-9);
    assert!((n.get(0) - 0.6_f64).abs() < 1e-9);
    assert!((n.get(1) - 0.8_f64).abs() < 1e-9);
}

#[test]
fn vector_normalize_zero_norm() {
    let v = Vector::from_slice(&[0.0_f64, 0.0]);
    let n = v.normalize();
    assert_eq!(n.rows(), 2);
    let a: f64 = n.get(0);
    let b: f64 = n.get(1);
    assert!(a.abs() < 1e-20);
    assert!(b.abs() < 1e-20);
}

#[test]
fn vector_display() {
    let v = Vector::from_slice(&[1.0_f64, 2.0, 3.0]);
    let s = format!("{}", v);
    assert!(s.contains("Vector"));
    assert!(s.contains("1"));
    assert!(s.contains("2"));
    assert!(s.contains("3"));
}

/// CpuExecutor.dot matches Vector::dot when both use CPU (f32).
#[test]
fn executor_dot_matches_vector_dot() {
    let x = Vector::from_slice(&[1.0_f32, 2.0, 3.0, 4.0]);
    let y = Vector::from_slice(&[0.5_f32, 1.0, 1.5, 2.0]);
    let dot_vec = x.dot(&y);
    let dot_exec = CpuExecutor.dot(&x, &y);
    assert!(
        (dot_vec - dot_exec).abs() < 1e-5,
        "CpuExecutor.dot should match Vector::dot: vec={} exec={}",
        dot_vec,
        dot_exec
    );
}

/// f64 Vector::dot uses CPU (SIMD when `simd` feature on). Verifies correctness.
#[test]
fn vector_f64_dot() {
    let a = Vector::from_slice(&[1.0_f64, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]);
    let b = Vector::from_slice(&[2.0_f64, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0]);
    let dot = a.dot(&b);
    let expected: f64 = a
        .data()
        .iter()
        .zip(b.data().iter())
        .map(|(x, y)| x * y)
        .sum();
    assert!(
        (dot - expected).abs() < 1e-10,
        "f64 dot: got {} expected {}",
        dot,
        expected
    );
}

#[test]
fn vector_index_and_index_mut() {
    let mut v = Vector::from_slice(&[1.0_f64, 2.0, 3.0]);
    assert!((v[0] - 1.0_f64).abs() < 1e-9);
    assert!((v[1] - 2.0_f64).abs() < 1e-9);
    v[0] = 10.0_f64;
    v[2] = 30.0_f64;
    assert!((v.get(0) - 10.0_f64).abs() < 1e-9);
    assert!((v.get(2) - 30.0_f64).abs() < 1e-9);
}
