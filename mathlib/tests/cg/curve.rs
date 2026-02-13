//! Unit tests for 3D curve evaluation (linear, Bezier, Hermite, B-spline).

use mathlib::math::curve::{bezier_curve, bspline_curve, hermite_curve, linear_curve};

const EPS: f32 = 1e-5;

fn assert_near(a: [f32; 3], b: [f32; 3], eps: f32) {
    assert!((a[0] - b[0]).abs() < eps, "x: {} vs {}", a[0], b[0]);
    assert!((a[1] - b[1]).abs() < eps, "y: {} vs {}", a[1], b[1]);
    assert!((a[2] - b[2]).abs() < eps, "z: {} vs {}", a[2], b[2]);
}

#[test]
fn linear_boundaries_and_mid() {
    let start = [0.0, 1.0, 2.0];
    let end = [3.0, 4.0, 5.0];
    assert_near(linear_curve(start, end, 0.0), start, EPS);
    assert_near(linear_curve(start, end, 1.0), end, EPS);
    let mid = linear_curve(start, end, 0.5);
    assert_near(mid, [1.5, 2.5, 3.5], EPS);
}

#[test]
fn bezier_endpoints() {
    let p0 = [0.0, 0.0, 0.0];
    let p1 = [1.0, 0.0, 0.0];
    let p2 = [1.0, 1.0, 0.0];
    let p3 = [1.0, 1.0, 1.0];
    assert_near(bezier_curve(p0, p1, p2, p3, 0.0), p0, EPS);
    assert_near(bezier_curve(p0, p1, p2, p3, 1.0), p3, EPS);
}

#[test]
fn hermite_zero_tangents_matches_linear() {
    let p0 = [0.0, 0.0, 0.0];
    let p1 = [2.0, 4.0, 6.0];
    let m0 = [0.0, 0.0, 0.0];
    let m1 = [0.0, 0.0, 0.0];
    assert_near(hermite_curve(p0, p1, m0, m1, 0.0), p0, EPS);
    assert_near(hermite_curve(p0, p1, m0, m1, 1.0), p1, EPS);
    assert_near(
        hermite_curve(p0, p1, m0, m1, 0.5),
        linear_curve(p0, p1, 0.5),
        EPS,
    );
}

#[test]
fn bspline_segment_bounds() {
    let p0 = [0.0, 0.0, 0.0];
    let p1 = [1.0, 0.0, 0.0];
    let p2 = [2.0, 0.0, 0.0];
    let p3 = [3.0, 0.0, 0.0];
    let v0 = bspline_curve(p0, p1, p2, p3, 0.0);
    let v1 = bspline_curve(p0, p1, p2, p3, 1.0);
    assert!(v0[0] >= 0.0 && v0[0] <= 3.0);
    assert!(v1[0] >= 0.0 && v1[0] <= 3.0);
}
