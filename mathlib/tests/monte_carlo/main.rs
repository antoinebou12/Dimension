//! Integration tests for Monte Carlo (estimate_pi, integrate_1d).

use mathlib::{estimate_pi, integrate_1d};
use std::f64::consts::PI;

#[test]
fn estimate_pi_same_seed_same_result() {
    let a = estimate_pi(42, 10_000);
    let b = estimate_pi(42, 10_000);
    assert_eq!(a, b);
}

#[test]
fn estimate_pi_converges_toward_pi() {
    let pi_est = estimate_pi(123, 500_000);
    assert!(
        (pi_est - PI).abs() < 0.01,
        "estimate {} should be within 0.01 of π",
        pi_est
    );
}

#[test]
#[should_panic(expected = "n_samples must be positive")]
fn estimate_pi_panics_zero_samples() {
    let _ = estimate_pi(42, 0);
}

#[test]
fn integrate_1d_same_seed_same_result() {
    let f = |x: f64| x * x;
    let a = integrate_1d(f, 0.0, 1.0, 10_000, 456);
    let b = integrate_1d(f, 0.0, 1.0, 10_000, 456);
    assert_eq!(a, b);
}

#[test]
fn integrate_1d_x_squared() {
    // ∫₀¹ x² dx = 1/3
    let integral = integrate_1d(|x| x * x, 0.0, 1.0, 100_000, 789);
    let expected = 1.0 / 3.0;
    assert!(
        (integral - expected).abs() < 0.02,
        "integral {} should be within 0.02 of 1/3",
        integral
    );
}

#[test]
fn integrate_1d_constant_one() {
    // ∫₀¹ 1 dx = 1
    let integral = integrate_1d(|_| 1.0, 0.0, 1.0, 100_000, 111);
    assert!(
        (integral - 1.0).abs() < 0.02,
        "integral {} should be within 0.02 of 1",
        integral
    );
}

#[test]
#[should_panic(expected = "n_samples must be positive")]
fn integrate_1d_panics_zero_samples() {
    let _ = integrate_1d(|x| x, 0.0, 1.0, 0, 42);
}
