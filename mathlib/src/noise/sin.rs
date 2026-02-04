//! Sinusoidal / wave noise: deterministic 2D height from sin/cos.

use crate::trig::{cos_scalar, sin_scalar};
use crate::vector::RealNumber;

/// Wave height at (u, v) in [0, 1]². Returns value in [0, 1].
///
/// Uses `k1 = 4π`, `k2 = 6π`. Implemented with mathlib trig only.
#[inline]
pub fn wave_2d(u: f64, v: f64) -> f64 {
    wave_2d_params(u, v, 4.0 * f64::pi(), 6.0 * f64::pi())
}

/// Wave height with configurable wave numbers (radians per unit).
///
/// `h = 0.5 + 0.25*sin(k1*u)*cos(k1*v) + 0.25*cos(k2*u)*sin(k2*v)` clamped to [0, 1].
#[inline]
pub fn wave_2d_params(u: f64, v: f64, k1: f64, k2: f64) -> f64 {
    let h = 0.5
        + 0.25 * sin_scalar(k1 * u) * cos_scalar(k1 * v)
        + 0.25 * cos_scalar(k2 * u) * sin_scalar(k2 * v);
    (h.clamp(0.0, 1.0) * 255.0).round() / 255.0
}
