//! Integration tests for dual quaternions (rigid transforms).

use mathlib::cg::{transform_point, vector3};
use mathlib::{DualQuat4f, Quat4f};

const EPS: f32 = 1e-5;

#[test]
fn dual_quat_identity() {
    let dq = DualQuat4f::identity();
    let p = vector3(1.0, 2.0, 3.0);
    let out = dq.transform_point(&p);
    assert!((out.get(0) - 1.0).abs() < EPS);
    assert!((out.get(1) - 2.0).abs() < EPS);
    assert!((out.get(2) - 3.0).abs() < EPS);
    assert!(dq.is_unit(EPS));
}

#[test]
fn dual_quat_roundtrip_via_matrix() {
    let rot = Quat4f::from_axis_angle(&vector3(0.0, 1.0, 0.0), 0.5);
    let t = vector3(1.0, -2.0, 0.5);
    let dq = DualQuat4f::from_rotation_and_translation(&rot, &t);
    let m = dq.to_matrix4();
    let p = vector3(1.0, 0.0, 0.0);
    let out_dq = dq.transform_point(&p);
    let out_m = transform_point(&m, &p);
    assert!((out_dq.get(0) - out_m.get(0)).abs() < EPS);
    assert!((out_dq.get(1) - out_m.get(1)).abs() < EPS);
    assert!((out_dq.get(2) - out_m.get(2)).abs() < EPS);
}

#[test]
fn dual_quat_from_matrix4_roundtrip() {
    let rot = Quat4f::from_axis_angle(&vector3(1.0, 1.0, 0.0).normalize(), 0.3);
    let t = vector3(-1.0, 2.0, 0.0);
    let dq0 = DualQuat4f::from_rotation_and_translation(&rot, &t);
    let m = dq0.to_matrix4();
    let dq1 = DualQuat4f::from_matrix4(&m);
    let p = vector3(0.5, 0.5, 0.5);
    let out0 = dq0.transform_point(&p);
    let out1 = dq1.transform_point(&p);
    assert!((out0.get(0) - out1.get(0)).abs() < EPS);
    assert!((out0.get(1) - out1.get(1)).abs() < EPS);
    assert!((out0.get(2) - out1.get(2)).abs() < EPS);
}

#[test]
fn dual_quat_composition() {
    let rot_a = Quat4f::from_axis_angle(&vector3(0.0, 1.0, 0.0), 0.2);
    let t_a = vector3(1.0, 0.0, 0.0);
    let dq_a = DualQuat4f::from_rotation_and_translation(&rot_a, &t_a);
    let rot_b = Quat4f::from_axis_angle(&vector3(1.0, 0.0, 0.0), 0.1);
    let t_b = vector3(0.0, 1.0, 0.0);
    let dq_b = DualQuat4f::from_rotation_and_translation(&rot_b, &t_b);
    let p = vector3(0.0, 0.0, 0.0);
    let composed = dq_a * dq_b;
    let out_ab = composed.transform_point(&p);
    let out_b_then_a = dq_a.transform_point(&dq_b.transform_point(&p));
    assert!((out_ab.get(0) - out_b_then_a.get(0)).abs() < EPS);
    assert!((out_ab.get(1) - out_b_then_a.get(1)).abs() < EPS);
    assert!((out_ab.get(2) - out_b_then_a.get(2)).abs() < EPS);
}

#[test]
fn dual_quat_inverse() {
    let rot = Quat4f::from_axis_angle(&vector3(0.0, 1.0, 0.0), 0.7);
    let t = vector3(2.0, -1.0, 0.5);
    let dq = DualQuat4f::from_rotation_and_translation(&rot, &t);
    let inv = dq.inverse();
    let p = vector3(1.0, 2.0, 3.0);
    let back = (dq * inv).transform_point(&p);
    assert!((back.get(0) - p.get(0)).abs() < EPS);
    assert!((back.get(1) - p.get(1)).abs() < EPS);
    assert!((back.get(2) - p.get(2)).abs() < EPS);
}

#[test]
fn dual_quat_pure_translation() {
    let rot = Quat4f::identity();
    let t = vector3(10.0, -5.0, 0.0);
    let dq = DualQuat4f::from_rotation_and_translation(&rot, &t);
    let p = vector3(1.0, 0.0, 0.0);
    let out = dq.transform_point(&p);
    assert!((out.get(0) - 11.0).abs() < EPS);
    assert!((out.get(1) + 5.0).abs() < EPS);
    assert!(out.get(2).abs() < EPS);
}

#[test]
fn dual_quat_pure_rotation() {
    let rot = Quat4f::from_axis_angle(&vector3(0.0, 1.0, 0.0), std::f32::consts::FRAC_PI_2);
    let t = vector3(0.0, 0.0, 0.0);
    let dq = DualQuat4f::from_rotation_and_translation(&rot, &t);
    let p = vector3(1.0, 0.0, 0.0);
    let out = dq.transform_point(&p);
    assert!(out.get(0).abs() < EPS);
    assert!(out.get(1).abs() < EPS);
    assert!((out.get(2) + 1.0).abs() < EPS);
}

#[test]
fn dual_quat_norm2_unit() {
    let rot = Quat4f::identity();
    let t = vector3(1.0, 0.0, 0.0);
    let dq = DualQuat4f::from_rotation_and_translation(&rot, &t);
    let (real, dual) = dq.norm2();
    assert!((real - 1.0).abs() < EPS);
    assert!(dual.abs() < EPS);
}

#[test]
fn dual_quat_batch_matches_single() {
    let rot = Quat4f::from_axis_angle(&vector3(0.0, 1.0, 0.0), 0.4);
    let t = vector3(1.0, -1.0, 0.5);
    let dq = DualQuat4f::from_rotation_and_translation(&rot, &t);
    let points = [
        vector3(0.0, 0.0, 0.0),
        vector3(1.0, 0.0, 0.0),
        vector3(0.0, 1.0, 0.0),
        vector3(1.0, 1.0, 1.0),
    ];
    let batch = dq.batch_transform_points(&points);
    for (i, p) in points.iter().enumerate() {
        let single = dq.transform_point(p);
        assert!((batch[i].get(0) - single.get(0)).abs() < EPS);
        assert!((batch[i].get(1) - single.get(1)).abs() < EPS);
        assert!((batch[i].get(2) - single.get(2)).abs() < EPS);
    }
}
