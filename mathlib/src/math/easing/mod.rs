//! Easing and interpolation functions for animation and smooth transitions.
//!
//! Parameter `t` is typically in the range `[0, 1]` (start to end). See [easings.net](https://easings.net) for visual references.
//!
//! For quaternion spherical linear interpolation, use [`Quat4f::slerp`](crate::math::quaternion::Quat4f::slerp).

use crate::vector::RealNumber;
use std::cmp::PartialOrd;
use std::ops::{Add, Neg, Sub};

pub use crate::math::quaternion::Quat4f;

/// Scalar type supporting the operations required for easing (`RealNumber` + powi + Neg + `PartialOrd`).
pub trait EasingScalar:
    Copy + RealNumber + Add<Output = Self> + Sub<Output = Self> + Neg<Output = Self> + PartialOrd
{
    /// Raises self to an integer power.
    fn powi_ease(self, n: i32) -> Self;
}

impl EasingScalar for f32 {
    fn powi_ease(self, n: i32) -> Self {
        self.powi(n)
    }
}

impl EasingScalar for f64 {
    fn powi_ease(self, n: i32) -> Self {
        self.powi(n)
    }
}

/// Converts to f64 for exp2 in expo easings (implemented for `f32` and `f64`).
pub trait ToF64 {
    fn to_f64(self) -> f64;
}
impl ToF64 for f32 {
    fn to_f64(self) -> f64 {
        f64::from(self)
    }
}
impl ToF64 for f64 {
    fn to_f64(self) -> f64 {
        self
    }
}

#[inline]
fn exp2_approx<T: EasingScalar + ToF64>(t: T, scale: f64, shift: f64) -> T {
    let x = t.to_f64() * scale + shift;
    T::from_f64(2f64.powf(x))
}

// --- Linear ---

/// Identity: returns `t` unchanged. Useful as a no-op easing.
///
/// # Example
///
/// ```
/// use mathlib::easing::linear;
/// assert!((linear(0.5f64) - 0.5).abs() < 1e-10);
/// assert_eq!(linear(0.0), 0.0);
/// assert_eq!(linear(1.0), 1.0);
/// ```
#[must_use]
pub fn linear<T: Copy>(t: T) -> T {
    t
}

/// Linear interpolation: `a + t * (b - a)`. For `t` in [0, 1], result is between `a` and `b`.
///
/// # Example
///
/// ```
/// use mathlib::easing::lerp;
/// assert!((lerp(0.0f64, 10.0, 0.5) - 5.0).abs() < 1e-10);
/// assert_eq!(lerp(0.0, 10.0, 0.0), 0.0);
/// assert_eq!(lerp(0.0, 10.0, 1.0), 10.0);
/// ```
#[must_use]
pub fn lerp<T>(a: T, b: T, t: T) -> T
where
    T: Copy + Add<Output = T> + Sub<Output = T> + std::ops::Mul<Output = T>,
{
    a + (b - a) * t
}

// --- Sine ---

/// Ease-in sine: `1 - cos(t * π/2)`.
#[must_use]
pub fn ease_in_sine<T: EasingScalar>(t: T) -> T {
    let one = T::from_f64(1.0);
    let half_pi = T::pi() * T::from_f64(0.5);
    one - (t * half_pi).cos()
}

/// Ease-out sine: `sin(t * π/2)`.
#[must_use]
pub fn ease_out_sine<T: EasingScalar>(t: T) -> T {
    let half_pi = T::pi() * T::from_f64(0.5);
    (t * half_pi).sin()
}

/// Ease-in-out sine: `(1 - cos(π*t)) / 2`.
#[must_use]
pub fn ease_in_out_sine<T: EasingScalar>(t: T) -> T {
    let one = T::from_f64(1.0);
    let two = T::from_f64(2.0);
    (one - (t * T::pi()).cos()) / two
}

// --- Quad ---

/// Ease-in quad: `t²`.
#[must_use]
pub fn ease_in_quad<T: EasingScalar>(t: T) -> T {
    t.powi_ease(2)
}

/// Ease-out quad: `1 - (1-t)²`.
#[must_use]
pub fn ease_out_quad<T: EasingScalar>(t: T) -> T {
    let one = T::from_f64(1.0);
    one - (one - t).powi_ease(2)
}

/// Ease-in-out quad.
#[must_use]
pub fn ease_in_out_quad<T: EasingScalar>(t: T) -> T {
    let one = T::from_f64(1.0);
    let two = T::from_f64(2.0);
    if t < T::from_f64(0.5) {
        two * t.powi_ease(2)
    } else {
        one - (-two * t + two).powi_ease(2) / two
    }
}

// --- Cubic ---

/// Ease-in cubic: `t³`.
#[must_use]
pub fn ease_in_cubic<T: EasingScalar>(t: T) -> T {
    t.powi_ease(3)
}

/// Ease-out cubic: `1 - (1-t)³`.
#[must_use]
pub fn ease_out_cubic<T: EasingScalar>(t: T) -> T {
    let one = T::from_f64(1.0);
    one - (one - t).powi_ease(3)
}

/// Ease-in-out cubic.
#[must_use]
pub fn ease_in_out_cubic<T: EasingScalar>(t: T) -> T {
    let one = T::from_f64(1.0);
    let two = T::from_f64(2.0);
    let four = T::from_f64(4.0);
    if t < T::from_f64(0.5) {
        four * t.powi_ease(3)
    } else {
        one - (-two * t + two).powi_ease(3) / two
    }
}

// --- Quart ---

/// Ease-in quart: `t⁴`.
#[must_use]
pub fn ease_in_quart<T: EasingScalar>(t: T) -> T {
    t.powi_ease(4)
}

/// Ease-out quart: `1 - (1-t)⁴`.
#[must_use]
pub fn ease_out_quart<T: EasingScalar>(t: T) -> T {
    let one = T::from_f64(1.0);
    one - (one - t).powi_ease(4)
}

/// Ease-in-out quart.
#[must_use]
pub fn ease_in_out_quart<T: EasingScalar>(t: T) -> T {
    let one = T::from_f64(1.0);
    let two = T::from_f64(2.0);
    let eight = T::from_f64(8.0);
    if t < T::from_f64(0.5) {
        eight * t.powi_ease(4)
    } else {
        one - (-two * t + two).powi_ease(4) / two
    }
}

// --- Quint ---

/// Ease-in quint: `t⁵`.
#[must_use]
pub fn ease_in_quint<T: EasingScalar>(t: T) -> T {
    t.powi_ease(5)
}

/// Ease-out quint: `1 - (1-t)⁵`.
#[must_use]
pub fn ease_out_quint<T: EasingScalar>(t: T) -> T {
    let one = T::from_f64(1.0);
    one - (one - t).powi_ease(5)
}

/// Ease-in-out quint.
#[must_use]
pub fn ease_in_out_quint<T: EasingScalar>(t: T) -> T {
    let one = T::from_f64(1.0);
    let two = T::from_f64(2.0);
    let sixteen = T::from_f64(16.0);
    if t < T::from_f64(0.5) {
        sixteen * t.powi_ease(5)
    } else {
        one - (-two * t + two).powi_ease(5) / two
    }
}

// --- Expo ---

/// Ease-in expo: `t == 0` then 0, else `2^(10*t - 10)`.
#[must_use]
pub fn ease_in_expo<T: EasingScalar + ToF64>(t: T) -> T {
    let zero = T::from_f64(0.0);
    let one = T::from_f64(1.0);
    if t == zero {
        return zero;
    }
    if t == one {
        return one;
    }
    exp2_approx(t, 10.0, -10.0)
}

/// Ease-out expo: `t == 1` then 1, else `1 - 2^(-10*t)`.
#[must_use]
pub fn ease_out_expo<T: EasingScalar + ToF64>(t: T) -> T {
    let zero = T::from_f64(0.0);
    let one = T::from_f64(1.0);
    if t == zero {
        return zero;
    }
    if t == one {
        return one;
    }
    one - exp2_approx(t, -10.0, 0.0)
}

/// Ease-in-out expo.
#[must_use]
pub fn ease_in_out_expo<T: EasingScalar + ToF64>(t: T) -> T {
    let zero = T::from_f64(0.0);
    let one = T::from_f64(1.0);
    let half = T::from_f64(0.5);
    let two = T::from_f64(2.0);
    if t == zero {
        return zero;
    }
    if t == one {
        return one;
    }
    if t < half {
        exp2_approx(t, 20.0, -10.0) / two
    } else {
        (two - exp2_approx(t, -20.0, 10.0)) / two
    }
}

// --- Circ ---

/// Ease-in circ: `1 - sqrt(1 - t²)`.
#[must_use]
pub fn ease_in_circ<T: EasingScalar>(t: T) -> T {
    let one = T::from_f64(1.0);
    one - (one - t.powi_ease(2)).sqrt()
}

/// Ease-out circ: `sqrt(1 - (t-1)²)`.
#[must_use]
pub fn ease_out_circ<T: EasingScalar>(t: T) -> T {
    let one = T::from_f64(1.0);
    (one - (t - one).powi_ease(2)).sqrt()
}

/// Ease-in-out circ.
#[must_use]
pub fn ease_in_out_circ<T: EasingScalar>(t: T) -> T {
    let one = T::from_f64(1.0);
    let two = T::from_f64(2.0);
    let half = T::from_f64(0.5);
    if t < half {
        (one - (one - (two * t).powi_ease(2)).sqrt()) / two
    } else {
        ((one - (-two * t + two).powi_ease(2)).sqrt() + one) / two
    }
}

// --- Back (overshoot constant C ≈ 1.70158) ---

const BACK_C: f64 = 1.70158;

/// Ease-in back: overshoot at start.
#[must_use]
pub fn ease_in_back<T: EasingScalar>(t: T) -> T {
    let c = T::from_f64(BACK_C);
    let one = T::from_f64(1.0);
    (c + one) * t.powi_ease(3) - c * t.powi_ease(2)
}

/// Ease-out back: overshoot at end.
#[must_use]
pub fn ease_out_back<T: EasingScalar>(t: T) -> T {
    let c = T::from_f64(BACK_C);
    let one = T::from_f64(1.0);
    let u = t - one;
    one + (c + one) * u.powi_ease(3) + c * u.powi_ease(2)
}

/// Ease-in-out back.
#[must_use]
pub fn ease_in_out_back<T: EasingScalar>(t: T) -> T {
    let c = T::from_f64(BACK_C * 1.525);
    let one = T::from_f64(1.0);
    let two = T::from_f64(2.0);
    let half = T::from_f64(0.5);
    if t < half {
        ((two * t).powi_ease(2) * (c + one) - c * (two * t)) / two
    } else {
        ((two * t - two).powi_ease(2) * (c + one) + c * (two * t - two)) / two + one
    }
}

// --- Elastic ---

/// Ease-in elastic.
#[must_use]
pub fn ease_in_elastic<T: EasingScalar + ToF64>(t: T) -> T {
    let zero = T::from_f64(0.0);
    let one = T::from_f64(1.0);
    if t == zero {
        return zero;
    }
    if t == one {
        return one;
    }
    let c4 = 2.0 * std::f64::consts::FRAC_PI_3;
    let angle = (t.to_f64() * 20.0 - 11.125) * c4;
    -T::from_f64(2f64.powf(20.0 * t.to_f64() - 10.0)) * T::from_f64(angle).sin()
}

/// Ease-out elastic.
#[must_use]
pub fn ease_out_elastic<T: EasingScalar + ToF64>(t: T) -> T {
    let zero = T::from_f64(0.0);
    let one = T::from_f64(1.0);
    if t == zero {
        return zero;
    }
    if t == one {
        return one;
    }
    let c4 = 2.0 * std::f64::consts::FRAC_PI_3;
    let angle = (t.to_f64() * 20.0 - 11.125) * c4;
    one + T::from_f64(2f64.powf(-20.0 * t.to_f64()) * angle.sin() * c4)
}

/// Ease-in-out elastic.
#[must_use]
pub fn ease_in_out_elastic<T: EasingScalar + ToF64>(t: T) -> T {
    let zero = T::from_f64(0.0);
    let one = T::from_f64(1.0);
    let half = T::from_f64(0.5);
    let two = T::from_f64(2.0);
    if t == zero {
        return zero;
    }
    if t == one {
        return one;
    }
    let c4 = 2.0 * std::f64::consts::FRAC_PI_3;
    if t < half {
        let angle = (t.to_f64() * 20.0 - 11.125) * c4;
        -T::from_f64(2f64.powf(20.0 * t.to_f64() - 10.0) * angle.sin() * c4) / two
    } else {
        let angle = (t.to_f64() * 20.0 - 11.125) * c4;
        (T::from_f64(2f64.powf(-20.0 * t.to_f64() + 10.0) * angle.sin() * c4) + two) / two
    }
}

// --- Bounce ---

fn ease_out_bounce_impl<T: EasingScalar>(t: T) -> T {
    let n1 = T::from_f64(7.5625);
    let d1 = T::from_f64(2.75);
    let one = T::from_f64(1.0);
    if t < one / d1 {
        n1 * t.powi_ease(2)
    } else if t < T::from_f64(2.0) / d1 {
        let t = t - T::from_f64(1.5) / d1;
        n1 * t.powi_ease(2) + T::from_f64(0.75)
    } else if t < T::from_f64(2.5) / d1 {
        let t = t - T::from_f64(2.25) / d1;
        n1 * t.powi_ease(2) + T::from_f64(0.9375)
    } else {
        let t = t - T::from_f64(2.625) / d1;
        n1 * t.powi_ease(2) + T::from_f64(0.984_375)
    }
}

/// Ease-out bounce.
#[must_use]
pub fn ease_out_bounce<T: EasingScalar>(t: T) -> T {
    ease_out_bounce_impl(t)
}

/// Ease-in bounce.
#[must_use]
pub fn ease_in_bounce<T: EasingScalar>(t: T) -> T {
    let one = T::from_f64(1.0);
    one - ease_out_bounce_impl(one - t)
}

/// Ease-in-out bounce.
#[must_use]
pub fn ease_in_out_bounce<T: EasingScalar>(t: T) -> T {
    let one = T::from_f64(1.0);
    let two = T::from_f64(2.0);
    let half = T::from_f64(0.5);
    if t < half {
        (one - ease_out_bounce_impl(one - two * t)) / two
    } else {
        (ease_out_bounce_impl(two * t - one) + one) / two
    }
}

// --- Hermite ---

/// Cubic Hermite interpolation: `p0`, `p1` are values at t=0 and t=1; `m0`, `m1` are tangents.
///
/// # Example
///
/// ```
/// use mathlib::easing::hermite;
/// // Linear segment from 0 to 1 with zero tangents
/// assert!((hermite(0.0f64, 1.0, 0.0, 0.0, 0.5) - 0.5).abs() < 1e-10);
/// ```
#[must_use]
pub fn hermite<T: EasingScalar>(p0: T, p1: T, m0: T, m1: T, t: T) -> T {
    let one = T::from_f64(1.0);
    let two = T::from_f64(2.0);
    let three = T::from_f64(3.0);
    let t2 = t.powi_ease(2);
    let t3 = t.powi_ease(3);
    let h00 = two * t3 - three * t2 + one;
    let h10 = t3 - two * t2 + t;
    let h01 = -two * t3 + three * t2;
    let h11 = t3 - t2;
    h00 * p0 + h10 * m0 + h01 * p1 + h11 * m1
}

// --- B-spline ---

/// Cubic B-spline segment: single segment with 4 control points, `t` in [0, 1].
///
/// # Example
///
/// ```
/// use mathlib::easing::bspline;
/// let pts = [0.0f64, 1.0, 2.0, 3.0];
/// let mid = bspline(&pts, 0.5);
/// assert!((mid - 1.5).abs() < 0.1);
/// ```
#[must_use]
pub fn bspline<T: EasingScalar>(control_points: &[T; 4], t: T) -> T {
    let one = T::from_f64(1.0);
    let three = T::from_f64(3.0);
    let six = T::from_f64(6.0);
    let u = t;
    let u2 = u.powi_ease(2);
    let u3 = u.powi_ease(3);
    let b0 = (one - u).powi_ease(3) / six;
    let b1 = (three * u3 - T::from_f64(6.0) * u2 + T::from_f64(4.0)) / six;
    let b2 = (-three * u3 + three * u2 + three * u + one) / six;
    let b3 = u3 / six;
    b0 * control_points[0]
        + b1 * control_points[1]
        + b2 * control_points[2]
        + b3 * control_points[3]
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f64 = 1e-10;

    fn assert_near(a: f64, b: f64, eps: f64) {
        assert!((a - b).abs() < eps, "{} vs {}", a, b);
    }

    #[test]
    fn linear_boundaries_and_mid() {
        assert_near(linear(0.0f64), 0.0, EPS);
        assert_near(linear(1.0f64), 1.0, EPS);
        assert_near(linear(0.5f64), 0.5, EPS);
    }

    #[test]
    fn lerp_boundaries_and_mid() {
        assert_near(lerp(0.0, 10.0, 0.0), 0.0, EPS);
        assert_near(lerp(0.0, 10.0, 1.0), 10.0, EPS);
        assert_near(lerp(0.0, 10.0, 0.5), 5.0, EPS);
    }

    #[test]
    fn hermite_zero_tangents() {
        assert_near(hermite(0.0, 1.0, 0.0, 0.0, 0.0), 0.0, EPS);
        assert_near(hermite(0.0, 1.0, 0.0, 0.0, 1.0), 1.0, EPS);
        assert_near(hermite(0.0, 1.0, 0.0, 0.0, 0.5), 0.5, EPS);
    }

    #[test]
    fn bspline_symmetric() {
        let pts = [0.0f64, 1.0, 2.0, 3.0];
        let v0 = bspline(&pts, 0.0);
        let v1 = bspline(&pts, 1.0);
        assert!(
            (0.0..=2.0).contains(&v0),
            "bspline(0) should be in [0,2], got {}",
            v0
        );
        assert!(
            (1.0..=3.0).contains(&v1),
            "bspline(1) should be in [1,3], got {}",
            v1
        );
        let mid = bspline(&pts, 0.5);
        assert!(
            mid > 1.0 && mid < 2.0,
            "bspline(0.5) should be between 1 and 2, got {}",
            mid
        );
    }

    #[test]
    fn ease_in_sine_boundaries() {
        assert_near(ease_in_sine(0.0f64), 0.0, EPS);
        assert_near(ease_in_sine(1.0f64), 1.0, EPS);
    }

    #[test]
    fn ease_out_sine_boundaries() {
        assert_near(ease_out_sine(0.0f64), 0.0, EPS);
        assert_near(ease_out_sine(1.0f64), 1.0, EPS);
    }

    #[test]
    fn ease_in_out_sine_boundaries() {
        assert_near(ease_in_out_sine(0.0f64), 0.0, EPS);
        assert_near(ease_in_out_sine(1.0f64), 1.0, EPS);
    }

    #[test]
    fn ease_in_quad_boundaries() {
        assert_near(ease_in_quad(0.0f64), 0.0, EPS);
        assert_near(ease_in_quad(1.0f64), 1.0, EPS);
    }

    #[test]
    fn ease_out_quad_boundaries() {
        assert_near(ease_out_quad(0.0f64), 0.0, EPS);
        assert_near(ease_out_quad(1.0f64), 1.0, EPS);
    }

    #[test]
    fn ease_in_cubic_boundaries() {
        assert_near(ease_in_cubic(0.0f64), 0.0, EPS);
        assert_near(ease_in_cubic(1.0f64), 1.0, EPS);
    }

    #[test]
    fn ease_out_cubic_boundaries() {
        assert_near(ease_out_cubic(0.0f64), 0.0, EPS);
        assert_near(ease_out_cubic(1.0f64), 1.0, EPS);
    }

    #[test]
    fn ease_in_out_cubic_monotonic() {
        let steps: [f64; 11] = [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0];
        for i in 1..steps.len() {
            assert!(
                ease_in_out_cubic(steps[i]) >= ease_in_out_cubic(steps[i - 1]) - EPS,
                "monotonic at {} vs {}",
                steps[i],
                steps[i - 1]
            );
        }
    }

    #[test]
    fn ease_in_expo_boundaries() {
        assert_near(ease_in_expo(0.0f64), 0.0, EPS);
        assert_near(ease_in_expo(1.0f64), 1.0, EPS);
    }

    #[test]
    fn ease_out_bounce_boundaries() {
        assert_near(ease_out_bounce(0.0f64), 0.0, EPS);
        assert!(ease_out_bounce(1.0f64) >= 0.99);
    }
}
