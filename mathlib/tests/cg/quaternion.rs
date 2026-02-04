use mathlib::{Quat4f, Vector3f};

fn vector3(x: f32, y: f32, z: f32) -> Vector3f {
    let mut v = Vector3f::with_capacity(3);
    v.set(0, x);
    v.set(1, y);
    v.set(2, z);
    v
}

#[test]
fn quat_identity() {
    let q = Quat4f::identity();
    assert!((q.w - 1.0).abs() < 1e-6);
    assert!(q.x.abs() < 1e-6);
    assert!(q.y.abs() < 1e-6);
    assert!(q.z.abs() < 1e-6);
}

#[test]
fn quat_from_axis_angle_zero_angle() {
    let axis = vector3(1.0, 0.0, 0.0);
    let q = Quat4f::from_axis_angle(&axis, 0.0);
    assert!((q.w - 1.0).abs() < 1e-5);
    assert!(q.x.abs() < 1e-5);
    assert!(q.y.abs() < 1e-5);
    assert!(q.z.abs() < 1e-5);
}

#[test]
fn quat_conjugate_times_self_is_identity_norm() {
    let axis = vector3(1.0, 1.0, 0.0);
    let axis_n = axis.normalize();
    let q = Quat4f::from_axis_angle(&axis_n, 0.5);
    let qc = q.conjugate();
    let p = q * qc;
    assert!((p.w - 1.0).abs() < 1e-5);
    assert!(p.x.abs() < 1e-5);
    assert!(p.y.abs() < 1e-5);
    assert!(p.z.abs() < 1e-5);
}

#[test]
fn quat_to_rotation_matrix3_roundtrip() {
    let axis = vector3(1.0, 0.0, 0.0);
    let q = Quat4f::from_axis_angle(&axis, std::f32::consts::FRAC_PI_2);
    let r = q.to_rotation_matrix3();
    assert_eq!(r.rows(), 3);
    assert_eq!(r.cols(), 3);
    let rt_r = &r.transpose() * &r;
    for i in 0..3 {
        for j in 0..3 {
            let expected = if i == j { 1.0 } else { 0.0 };
            assert!((rt_r.get(i, j) - expected).abs() < 1e-5);
        }
    }
}

#[test]
fn quat_to_rotation_matrix4_has_identity_bottom_right() {
    let axis = vector3(0.0, 1.0, 0.0);
    let q = Quat4f::from_axis_angle(&axis, 0.3);
    let m = q.to_rotation_matrix4();
    assert!((m.get(3, 3) - 1.0).abs() < 1e-6);
    assert!(m.get(3, 0).abs() < 1e-6);
    assert!(m.get(3, 1).abs() < 1e-6);
    assert!(m.get(3, 2).abs() < 1e-6);
    assert!(m.get(0, 3).abs() < 1e-6);
    assert!(m.get(1, 3).abs() < 1e-6);
    assert!(m.get(2, 3).abs() < 1e-6);
}
