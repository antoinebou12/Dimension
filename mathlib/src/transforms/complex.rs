//! Complex number type for FFT and related transforms.
//!
//! A minimal, idiomatic `Complex64` type compatible with num-complex style usage.

use std::ops::{Add, Div, Mul, Neg, Sub};

/// Complex number with `f64` real and imaginary parts.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Complex64 {
    /// Real part.
    pub re: f64,
    /// Imaginary part.
    pub im: f64,
}

impl Complex64 {
    /// Creates a new complex number from real and imaginary parts.
    #[inline]
    pub const fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }

    /// Returns the complex conjugate: `re - i*im`.
    #[inline]
    pub fn conjugate(self) -> Self {
        Self {
            re: self.re,
            im: -self.im,
        }
    }

    /// Squared magnitude: `re^2 + im^2`.
    #[inline]
    pub fn norm_sqr(self) -> f64 {
        self.re * self.re + self.im * self.im
    }

    /// Magnitude (Euclidean norm).
    #[inline]
    pub fn norm(self) -> f64 {
        self.norm_sqr().sqrt()
    }

    /// Phase (argument) in radians, in [-π, π].
    #[inline]
    pub fn arg(self) -> f64 {
        f64::atan2(self.im, self.re)
    }

    /// Euler's formula: `cos(theta) + i*sin(theta)`.
    #[inline]
    pub fn exp_i(theta: f64) -> Self {
        Self {
            re: theta.cos(),
            im: theta.sin(),
        }
    }

    /// Creates a complex number from polar coordinates: `r * exp(i*theta)`.
    #[inline]
    pub fn from_polar(r: f64, theta: f64) -> Self {
        Self {
            re: r * theta.cos(),
            im: r * theta.sin(),
        }
    }

    /// Imaginary unit.
    pub const I: Self = Self { re: 0.0, im: 1.0 };

    /// Zero.
    pub const ZERO: Self = Self { re: 0.0, im: 0.0 };

    /// One.
    pub const ONE: Self = Self { re: 1.0, im: 0.0 };
}

impl From<f64> for Complex64 {
    #[inline]
    fn from(re: f64) -> Self {
        Self { re, im: 0.0 }
    }
}

impl From<(f64, f64)> for Complex64 {
    #[inline]
    fn from((re, im): (f64, f64)) -> Self {
        Self { re, im }
    }
}

impl Add for Complex64 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self {
            re: self.re + rhs.re,
            im: self.im + rhs.im,
        }
    }
}

impl Add<f64> for Complex64 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: f64) -> Self {
        Self {
            re: self.re + rhs,
            im: self.im,
        }
    }
}

impl Sub for Complex64 {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Self {
            re: self.re - rhs.re,
            im: self.im - rhs.im,
        }
    }
}

impl Sub<f64> for Complex64 {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: f64) -> Self {
        Self {
            re: self.re - rhs,
            im: self.im,
        }
    }
}

impl Mul for Complex64 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self {
        Self {
            re: self.re * rhs.re - self.im * rhs.im,
            im: self.re * rhs.im + self.im * rhs.re,
        }
    }
}

impl Mul<f64> for Complex64 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: f64) -> Self {
        Self {
            re: self.re * rhs,
            im: self.im * rhs,
        }
    }
}

impl Div for Complex64 {
    type Output = Self;
    #[inline]
    fn div(self, rhs: Self) -> Self {
        let denom = rhs.norm_sqr();
        Self {
            re: (self.re * rhs.re + self.im * rhs.im) / denom,
            im: (self.im * rhs.re - self.re * rhs.im) / denom,
        }
    }
}

impl Div<f64> for Complex64 {
    type Output = Self;
    #[inline]
    fn div(self, rhs: f64) -> Self {
        Self {
            re: self.re / rhs,
            im: self.im / rhs,
        }
    }
}

impl Neg for Complex64 {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Self {
            re: -self.re,
            im: -self.im,
        }
    }
}

impl std::fmt::Display for Complex64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.im >= 0.0 {
            write!(f, "{}+{}i", self.re, self.im)
        } else {
            write!(f, "{}-{}i", self.re, -self.im)
        }
    }
}
