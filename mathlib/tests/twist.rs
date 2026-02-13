use mathlib::cg::{from_scaled_axis, matrix4f_identity, new_translation, vector3};
use mathlib::{Matrix4f, Vector, Vector3f, clamp_twist, pose_twist_error};

fn identity_transform() -> Matrix4f {
    matrix4f_identity()
}

#[test]
fn pose_twist_error_zero() {
    let current = identity_transform();
    let target = identity_transform();
    let err = pose_twist_error(&current, &target);
    for i in 0..6 {
        assert!(err.get(i).abs() < 1e-6);
    }
}

#[test]
fn pose_twist_error_translation() {
    let current = identity_transform();
    let target = new_translation(&vector3(1.0, -2.0, 0.5));
    // Keep rotation identity
    let err = pose_twist_error(&current, &target);
    assert!((err.get(0) - 1.0).abs() < 1e-6);
    assert!((err.get(1) + 2.0).abs() < 1e-6);
    assert!((err.get(2) - 0.5).abs() < 1e-6);
    assert!(err.get(3).abs() < 1e-6);
    assert!(err.get(4).abs() < 1e-6);
    assert!(err.get(5).abs() < 1e-6);
}

#[test]
fn pose_twist_error_rotation() {
    let current = identity_transform();
    let axis = vector3(0.0, 0.0, 1.0);
    let mut target = from_scaled_axis(&axis_scale(&axis, std::f32::consts::FRAC_PI_2));
    // Ensure translation parts identical
    for i in 0..3 {
        target.set(i, 3, 0.0);
    }
    let err = pose_twist_error(&current, &target);
    assert!(err.get(0).abs() < 1e-5);
    assert!(err.get(1).abs() < 1e-5);
    assert!(err.get(2).abs() < 1e-5);
    assert!(err.get(3).abs() < 1e-4);
    assert!(err.get(4).abs() < 1e-4);
    assert!((err.get(5) - std::f32::consts::FRAC_PI_2).abs() < 1e-4);
}

#[test]
fn clamp_twist_limits_segments() {
    let mut step = Vector::with_capacity(6);
    step.set_zero();
    step.set(0, 10.0);
    step.set(1, 0.0);
    step.set(2, 0.0);
    step.set(3, 0.0);
    step.set(4, 0.0);
    step.set(5, 5.0);
    clamp_twist(&mut step, 1.0, 0.5);
    let lin_norm =
        (step.get(0) * step.get(0) + step.get(1) * step.get(1) + step.get(2) * step.get(2)).sqrt();
    let ang_norm =
        (step.get(3) * step.get(3) + step.get(4) * step.get(4) + step.get(5) * step.get(5)).sqrt();
    assert!((lin_norm - 1.0).abs() < 1e-5);
    assert!((ang_norm - 0.5).abs() < 1e-5);
}

fn axis_scale(axis: &Vector3f, angle: f32) -> Vector3f {
    let mut scaled = Vector3f::with_capacity(3);
    scaled.set(0, axis.get(0) * angle);
    scaled.set(1, axis.get(1) * angle);
    scaled.set(2, axis.get(2) * angle);
    scaled
}
