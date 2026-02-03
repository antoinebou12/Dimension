//! Component-wise trigonometric and angle-conversion functions for vectors (GLM-style).

use crate::vector::{RealNumber, Vector};
use std::ops::{Div, Mul};

/// Component-wise arc-cosinus.
pub fn acos<T: RealNumber>(x: &Vector<T>) -> Vector<T> {
    x.map(T::acos)
}

/// Component-wise hyperbolic arc-cosinus.
pub fn acosh<T: RealNumber>(x: &Vector<T>) -> Vector<T> {
    x.map(T::acosh)
}

/// Component-wise arc-sinus.
pub fn asin<T: RealNumber>(x: &Vector<T>) -> Vector<T> {
    x.map(T::asin)
}

/// Component-wise hyperbolic arc-sinus.
pub fn asinh<T: RealNumber>(x: &Vector<T>) -> Vector<T> {
    x.map(T::asinh)
}

/// Component-wise arc-tangent of `y / x`.
pub fn atan2<T: RealNumber>(y: &Vector<T>, x: &Vector<T>) -> Vector<T> {
    y.zip_map(x, T::atan2)
}

/// Component-wise arc-tangent.
pub fn atan<T: RealNumber>(y_over_x: &Vector<T>) -> Vector<T> {
    y_over_x.map(T::atan)
}

/// Component-wise hyperbolic arc-tangent.
pub fn atanh<T: RealNumber>(x: &Vector<T>) -> Vector<T> {
    x.map(T::atanh)
}

/// Scalar cosinus (convenience over a 1-element vector).
#[inline]
pub fn cos_scalar<T: RealNumber>(x: T) -> T {
    let mut v = Vector::with_capacity(1);
    v.set(0, x);
    cos(&v).get(0)
}

/// Component-wise cosinus.
pub fn cos<T: RealNumber>(angle: &Vector<T>) -> Vector<T> {
    angle.map(T::cos)
}

/// Component-wise hyperbolic cosinus.
pub fn cosh<T: RealNumber>(angle: &Vector<T>) -> Vector<T> {
    angle.map(T::cosh)
}

/// Component-wise conversion from radians to degrees.
pub fn degrees<T>(radians: &Vector<T>) -> Vector<T>
where
    T: RealNumber + Mul<Output = T> + Div<Output = T>,
{
    let one_eighty = T::from_f64(180.0);
    let pi = T::pi();
    radians.map(|e| e * one_eighty / pi)
}

/// Component-wise conversion from degrees to radians.
pub fn radians<T>(degrees: &Vector<T>) -> Vector<T>
where
    T: RealNumber + Mul<Output = T> + Div<Output = T>,
{
    let pi = T::pi();
    let one_eighty = T::from_f64(180.0);
    degrees.map(|e| e * pi / one_eighty)
}

/// Scalar sinus (convenience over a 1-element vector).
#[inline]
pub fn sin_scalar<T: RealNumber>(x: T) -> T {
    let mut v = Vector::with_capacity(1);
    v.set(0, x);
    sin(&v).get(0)
}

/// Component-wise sinus.
pub fn sin<T: RealNumber>(angle: &Vector<T>) -> Vector<T> {
    angle.map(T::sin)
}

/// Component-wise hyperbolic sinus.
pub fn sinh<T: RealNumber>(angle: &Vector<T>) -> Vector<T> {
    angle.map(T::sinh)
}

/// Component-wise tangent.
pub fn tan<T: RealNumber>(angle: &Vector<T>) -> Vector<T> {
    angle.map(T::tan)
}

/// Component-wise hyperbolic tangent.
pub fn tanh<T: RealNumber>(angle: &Vector<T>) -> Vector<T> {
    angle.map(T::tanh)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sin_cos() {
        let mut v: Vector<f64> = Vector::with_capacity(3);
        v.set(0, 0.0);
        v.set(1, std::f64::consts::FRAC_PI_2);
        v.set(2, std::f64::consts::PI);
        let s = sin(&v);
        let c = cos(&v);
        assert!((s.get(0) - 0.0).abs() < 1e-10);
        assert!((s.get(1) - 1.0).abs() < 1e-10);
        assert!((s.get(2)).abs() < 1e-10);
        assert!((c.get(0) - 1.0).abs() < 1e-10);
        assert!((c.get(1)).abs() < 1e-10);
        assert!((c.get(2) + 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_degrees_radians_roundtrip() {
        let mut v: Vector<f64> = Vector::with_capacity(2);
        v.set(0, 0.0);
        v.set(1, 180.0);
        let r = radians(&v);
        let d = degrees(&r);
        assert!((d.get(0) - 0.0).abs() < 1e-10);
        assert!((d.get(1) - 180.0).abs() < 1e-10);
    }

    #[test]
    fn test_atan2() {
        let mut y: Vector<f64> = Vector::with_capacity(2);
        let mut x: Vector<f64> = Vector::with_capacity(2);
        y.set(0, 1.0);
        y.set(1, 0.0);
        x.set(0, 1.0);
        x.set(1, 1.0);
        let a = atan2(&y, &x);
        assert!((a.get(0) - std::f64::consts::FRAC_PI_4).abs() < 1e-10);
        assert!((a.get(1) - 0.0).abs() < 1e-10);
    }
}
