//! Integration tests for noise (perlin, fbm, wave).

use mathlib::{fbm_2d, perlin_2d, wave_2d, wave_2d_params};

#[test]
fn perlin_2d_deterministic() {
    let a = perlin_2d(1.0, 2.0);
    let b = perlin_2d(1.0, 2.0);
    assert!((a - b).abs() < 1e-10);
    let a = perlin_2d(0.5, 0.5);
    let b = perlin_2d(0.5, 0.5);
    assert!((a - b).abs() < 1e-10);
}

#[test]
fn perlin_2d_bounded() {
    for (x, y) in [(0.0, 0.0), (1.5, 2.5), (10.0, 10.0)] {
        let v = perlin_2d(x, y);
        assert!(
            (-1.5..=1.5).contains(&v),
            "perlin_2d({}, {}) = {} out of range",
            x,
            y,
            v
        );
    }
}

#[test]
fn wave_2d_deterministic() {
    let a = wave_2d(0.3, 0.7);
    let b = wave_2d(0.3, 0.7);
    assert!((a - b).abs() < 1e-10);
}

#[test]
fn wave_2d_in_range() {
    assert!((0.0..=1.0).contains(&wave_2d(0.0, 0.0)));
    assert!((0.0..=1.0).contains(&wave_2d(0.5, 0.5)));
    assert!((0.0..=1.0).contains(&wave_2d(1.0, 1.0)));
}

#[test]
fn wave_2d_params_in_range() {
    assert!((0.0..=1.0).contains(&wave_2d_params(0.0, 0.0, 1.0, 2.0)));
}

#[test]
fn fbm_2d_deterministic() {
    let a = fbm_2d(1.0, 2.0, 4, 2.0, 0.5, perlin_2d);
    let b = fbm_2d(1.0, 2.0, 4, 2.0, 0.5, perlin_2d);
    assert!((a - b).abs() < 1e-10);
}
