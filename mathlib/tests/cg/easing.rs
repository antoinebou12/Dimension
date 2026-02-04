//! Integration tests for easing and interpolation (scalar easings, hermite, bspline, slerp).

use mathlib::easing::{
    bspline, ease_in_back, ease_in_bounce, ease_in_circ, ease_in_cubic, ease_in_elastic,
    ease_in_expo, ease_in_out_back, ease_in_out_bounce, ease_in_out_circ, ease_in_out_cubic,
    ease_in_out_elastic, ease_in_out_expo, ease_in_out_quad, ease_in_out_quart, ease_in_out_quint,
    ease_in_out_sine, ease_in_quad, ease_in_quart, ease_in_quint, ease_in_sine, ease_out_back,
    ease_out_bounce, ease_out_circ, ease_out_cubic, ease_out_elastic, ease_out_expo, ease_out_quad,
    ease_out_quart, ease_out_quint, ease_out_sine, hermite, lerp, linear,
};
use mathlib::{Quat4f, Vector3f};

const EPS: f64 = 1e-10;
const EPS_F32: f32 = 1e-5;

fn assert_near(a: f64, b: f64, eps: f64) {
    assert!((a - b).abs() < eps, "{} vs {}", a, b);
}

fn quat_dot(a: &Quat4f, b: &Quat4f) -> f32 {
    a.w * b.w + a.x * b.x + a.y * b.y + a.z * b.z
}

fn vector3(x: f32, y: f32, z: f32) -> Vector3f {
    let mut v = Vector3f::with_capacity(3);
    v.set(0, x);
    v.set(1, y);
    v.set(2, z);
    v
}

#[test]
fn linear_boundaries() {
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
    let mid = bspline(&pts, 0.5);
    assert!(
        mid > 1.0 && mid < 2.0,
        "bspline(0.5) should be between 1 and 2, got {}",
        mid
    );
}

#[test]
fn all_ease_in_boundaries() {
    let fns: [fn(f64) -> f64; 10] = [
        ease_in_sine,
        ease_in_quad,
        ease_in_cubic,
        ease_in_quart,
        ease_in_quint,
        ease_in_expo,
        ease_in_circ,
        ease_in_back,
        ease_in_elastic,
        ease_in_bounce,
    ];
    for f in fns {
        assert_near(f(0.0), 0.0, EPS);
        assert_near(f(1.0), 1.0, EPS);
    }
}

#[test]
fn all_ease_out_boundaries() {
    let fns: [fn(f64) -> f64; 10] = [
        ease_out_sine,
        ease_out_quad,
        ease_out_cubic,
        ease_out_quart,
        ease_out_quint,
        ease_out_expo,
        ease_out_circ,
        ease_out_back,
        ease_out_elastic,
        ease_out_bounce,
    ];
    for f in fns {
        assert_near(f(0.0), 0.0, EPS);
        assert_near(f(1.0), 1.0, EPS);
    }
}

#[test]
fn all_ease_in_out_boundaries() {
    let fns: [fn(f64) -> f64; 10] = [
        ease_in_out_sine,
        ease_in_out_quad,
        ease_in_out_cubic,
        ease_in_out_quart,
        ease_in_out_quint,
        ease_in_out_expo,
        ease_in_out_circ,
        ease_in_out_back,
        ease_in_out_elastic,
        ease_in_out_bounce,
    ];
    for f in fns {
        assert_near(f(0.0), 0.0, EPS);
        assert_near(f(1.0), 1.0, EPS);
    }
}

#[test]
fn monotonic_ease_in_cubic() {
    let steps: [f64; 11] = [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0];
    for i in 1..steps.len() {
        assert!(
            ease_in_cubic(steps[i]) >= ease_in_cubic(steps[i - 1]) - EPS,
            "monotonic at {} vs {}",
            steps[i],
            steps[i - 1]
        );
    }
}

#[test]
fn monotonic_ease_out_cubic() {
    let steps: [f64; 11] = [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0];
    for i in 1..steps.len() {
        assert!(
            ease_out_cubic(steps[i]) >= ease_out_cubic(steps[i - 1]) - EPS,
            "monotonic at {} vs {}",
            steps[i],
            steps[i - 1]
        );
    }
}

#[test]
fn slerp_identity() {
    let axis = vector3(0.0, 1.0, 0.0);
    let q = Quat4f::from_axis_angle(&axis, 0.5);
    let interp = q.slerp(&q, 0.3);
    assert!((interp.w - q.w).abs() < EPS_F32);
    assert!((interp.x - q.x).abs() < EPS_F32);
    assert!((interp.y - q.y).abs() < EPS_F32);
    assert!((interp.z - q.z).abs() < EPS_F32);
}

#[test]
fn slerp_endpoints() {
    let axis = vector3(1.0, 0.0, 0.0);
    let q0 = Quat4f::from_axis_angle(&axis, 0.0);
    let q1 = Quat4f::from_axis_angle(&axis, std::f32::consts::FRAC_PI_2);

    let at_0 = q0.slerp(&q1, 0.0);
    assert!((at_0.w - q0.w).abs() < EPS_F32);
    assert!((at_0.x - q0.x).abs() < EPS_F32);
    assert!((at_0.y - q0.y).abs() < EPS_F32);
    assert!((at_0.z - q0.z).abs() < EPS_F32);

    let at_1 = q0.slerp(&q1, 1.0);
    // Same rotation means dot product ≈ ±1 (q and -q are equivalent)
    let dot = quat_dot(&at_1, &q1);
    assert!(
        dot.abs() > 1.0 - EPS_F32,
        "slerp(1) should equal q1 (or -q1), dot={}",
        dot
    );
}

#[test]
fn slerp_interior_half_angle() {
    let axis = vector3(0.0, 1.0, 0.0);
    let q0 = Quat4f::from_axis_angle(&axis, 0.0);
    let q1 = Quat4f::from_axis_angle(&axis, std::f32::consts::FRAC_PI_2);

    let half = q0.slerp(&q1, 0.5);
    let expected = Quat4f::from_axis_angle(&axis, std::f32::consts::FRAC_PI_4);
    // Same rotation means dot product ≈ ±1
    let dot = quat_dot(&half, &expected);
    assert!(
        dot.abs() > 1.0 - EPS_F32,
        "slerp(0.5) should be half-angle rotation, dot={}",
        dot
    );
}
