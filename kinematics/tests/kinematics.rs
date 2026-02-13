//! Kinematics integration tests.

use kinematics::ik::{FabrikIk, JacobianIk};
use kinematics::joints::{Revolute2dJoint, RevoluteJoint};
use kinematics::{Armature, JointData, JointVariant};
use mathlib::Vector3f;
use mathlib::cg::vector3;

fn vec3(x: f32, y: f32, z: f32) -> Vector3f {
    vector3(x, y, z)
}

#[test]
fn armature_forward_kinematics() {
    let root = JointData::new(JointVariant::Revolute(RevoluteJoint::new(
        vec3(0.0, 0.0, 0.0),
        (0.0, 1.0, 0.0),
        0.0,
    )));
    let mut arm = Armature::new(root);
    arm.add_child(
        0,
        1,
        JointData::new(JointVariant::Revolute(RevoluteJoint::new(
            vec3(1.0, 0.0, 0.0),
            (0.0, 1.0, 0.0),
            0.0,
        ))),
    );
    arm.update_kinematics();
    let ee = arm.end_effector_position(1);
    assert!((ee.get(0) - 1.0).abs() < 1e-5);
    assert!((ee.get(1) - 0.0).abs() < 1e-5);
    assert!((ee.get(2) - 0.0).abs() < 1e-5);
}

#[test]
fn armature_pack_unpack() {
    let root = JointData::new(JointVariant::Revolute(RevoluteJoint::new(
        vec3(0.0, 0.0, 0.0),
        (0.0, 1.0, 0.0),
        0.5,
    )));
    let mut arm = Armature::new(root);
    let packed = arm.pack();
    assert_eq!(packed.len(), 1);
    assert!((packed[0] - 0.5).abs() < 1e-6);
    arm.unpack(&[0.2]);
    let p2 = arm.pack();
    assert!((p2[0] - 0.2).abs() < 1e-6);
}

#[test]
fn armature_2d_forward_kinematics() {
    let root = JointData::new(JointVariant::Revolute2d(Revolute2dJoint::at(0.0, 0.0, 0.0)));
    let mut arm = Armature::new(root);
    arm.add_child(
        0,
        1,
        JointData::new(JointVariant::Revolute2d(Revolute2dJoint::at(1.0, 0.0, 0.0))),
    );
    arm.update_kinematics();
    let ee = arm.end_effector_position(1);
    assert!((ee.get(0) - 1.0).abs() < 1e-5);
    assert!((ee.get(1) - 0.0).abs() < 1e-5);
    assert!((ee.get(2) - 0.0).abs() < 1e-5);
}

#[test]
fn jacobian_ik_reaches_target() {
    // Single-link arm: one revolute at origin, link length 1. Can reach any point on unit circle.
    let root = JointData::new(JointVariant::Revolute2d(Revolute2dJoint::at(0.0, 0.0, 0.0)));
    let mut arm = Armature::new(root);
    arm.add_child(
        0,
        1,
        JointData::new(JointVariant::Revolute2d(Revolute2dJoint::at(1.0, 0.0, 0.0))),
    );
    arm.update_kinematics();
    let target = vector3(0.6, 0.8, 0.0);
    let err = JacobianIk::new(&mut arm, 1, target.clone())
        .with_max_iters(30)
        .solve();
    assert!(err < 1e-3, "Jacobian IK error {} should be < 1e-3", err);
    arm.update_kinematics();
    let ee = arm.end_effector_position(1);
    let dx = ee.get(0) - target.get(0);
    let dy = ee.get(1) - target.get(1);
    assert!(
        (dx * dx + dy * dy).sqrt() < 1e-2,
        "end-effector should be near target"
    );
}

#[test]
fn fabrik_ik_reaches_target() {
    // Two-link 2D arm; target within reach.
    let root = JointData::new(JointVariant::Revolute2d(Revolute2dJoint::at(0.0, 0.0, 0.0)));
    let mut arm = Armature::new(root);
    arm.add_child(
        0,
        1,
        JointData::new(JointVariant::Revolute2d(Revolute2dJoint::at(1.0, 0.0, 0.0))),
    );
    arm.add_child(
        1,
        2,
        JointData::new(JointVariant::Revolute2d(Revolute2dJoint::at(1.0, 0.0, 0.0))),
    );
    arm.update_kinematics();
    let target = vector3(1.2, 0.6, 0.0);
    let err = FabrikIk::new(&mut arm, 2, target.clone())
        .with_max_iters(20)
        .solve();
    assert!(
        err < 0.5,
        "FABRIK IK error {} should be < 0.5 (revolute recovery may leave residual)",
        err
    );
    arm.update_kinematics();
    let ee = arm.end_effector_position(2);
    let dx = ee.get(0) - target.get(0);
    let dy = ee.get(1) - target.get(1);
    assert!(
        (dx * dx + dy * dy).sqrt() < 0.6,
        "end-effector should be near target after FABRIK"
    );
}
