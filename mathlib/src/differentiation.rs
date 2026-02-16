//! Numerical differentiation (Chapter 14).
//!
//! Finite-difference approximations for first and second derivatives.

/// Default step size for finite differences: h = ε^(1/3) * max(|x|, 1).
#[must_use]
pub fn default_step(x: f64) -> f64 {
    f64::EPSILON.cbrt() * x.abs().max(1.0)
}

/// Forward difference: f'(x) ≈ (f(x + h) - f(x)) / h.
#[must_use]
pub fn diff_forward<F>(f: F, x: f64, h: f64) -> f64
where
    F: Fn(f64) -> f64,
{
    (f(x + h) - f(x)) / h
}

/// Backward difference: f'(x) ≈ (f(x) - f(x - h)) / h.
#[must_use]
pub fn diff_backward<F>(f: F, x: f64, h: f64) -> f64
where
    F: Fn(f64) -> f64,
{
    (f(x) - f(x - h)) / h
}

/// Central difference: f'(x) ≈ (f(x + h) - f(x - h)) / (2h).
#[must_use]
pub fn diff_central<F>(f: F, x: f64, h: f64) -> f64
where
    F: Fn(f64) -> f64,
{
    (f(x + h) - f(x - h)) / (2.0 * h)
}

/// Second derivative (central): f''(x) ≈ (f(x + h) - 2f(x) + f(x - h)) / h².
#[must_use]
pub fn diff2_central<F>(f: F, x: f64, h: f64) -> f64
where
    F: Fn(f64) -> f64,
{
    (f(x + h) - 2.0 * f(x) + f(x - h)) / (h * h)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diff_central_x_squared() {
        let f = |x: f64| x * x;
        let h = 2.0_f64.powi(-17); // power-of-2 step avoids decimal repr issues
        let df = diff_central(f, 2.0, h);
        assert!((df - 4.0).abs() < 1e-8, "df = {}, expected 4.0", df);
    }

    #[test]
    fn diff2_central_x_squared() {
        let f = |x: f64| x * x;
        let h = 2.0_f64.powi(-17);
        let d2f = diff2_central(f, 2.0, h);
        assert!((d2f - 2.0).abs() < 1e-8, "d2f = {}, expected 2.0", d2f);
    }
}
