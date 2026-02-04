//! Integration tests for trig module: cos, sin, degrees, radians, scalar helpers.

use mathlib::{Vector, cos, cos_scalar, degrees, radians, sin, sin_scalar};

#[test]
fn trig_sin_cos() {
    let mut v = Vector::with_capacity(3);
    v.set(0, 0.0_f64);
    v.set(1, std::f64::consts::FRAC_PI_2);
    v.set(2, std::f64::consts::PI);
    let s = sin(&v);
    let c = cos(&v);
    assert!((s.get(0) - 0.0).abs() < 1e-10);
    assert!((s.get(1) - 1.0).abs() < 1e-10);
    assert!(s.get(2).abs() < 1e-10);
    assert!((c.get(0) - 1.0).abs() < 1e-10);
    assert!(c.get(1).abs() < 1e-10);
    assert!((c.get(2) + 1.0).abs() < 1e-10);
}

#[test]
fn trig_degrees_radians_roundtrip() {
    let mut v = Vector::with_capacity(2);
    v.set(0, 0.0_f64);
    v.set(1, 180.0);
    let r = radians(&v);
    let d = degrees(&r);
    assert!((d.get(0) - 0.0).abs() < 1e-10);
    assert!((d.get(1) - 180.0).abs() < 1e-10);
}

#[test]
fn trig_cos_scalar_sin_scalar() {
    let x = 0.0_f64;
    assert!((cos_scalar(x) - 1.0).abs() < 1e-10);
    assert!(sin_scalar(x).abs() < 1e-10);
    let x = std::f64::consts::FRAC_PI_2;
    assert!(cos_scalar(x).abs() < 1e-10);
    assert!((sin_scalar(x) - 1.0).abs() < 1e-10);
}
