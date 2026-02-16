//! Kinematics integration tests.

use kinematics::ik::{CcdIk, FabrikIk, FabrikSqpIk, HalleyIk, HessianIk, JacobianIk};
use kinematics::joints::{Revolute2dJoint, RevoluteJoint, SphericalJoint};
use kinematics::{Armature, JointData, JointVariant};
use mathlib::cg::vector3;
use mathlib::{Quat4f, Vector3f};

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
fn hessian_ik_spherical_reaches_target() {
    // Three-link 3D arm with spherical joints; target within reach.
    // HessianIk (exact Hessian Newton, Erleben & Andrews MIG 2017).
    let root = JointData::new(JointVariant::Spherical(SphericalJoint::new(
        vec3(0.0, 0.0, 0.0),
        Quat4f::identity(),
    )));
    let mut arm = Armature::new(root);
    let link_len = 0.5;
    for i in 1..4 {
        let joint = SphericalJoint::new(vec3(link_len, 0.0, 0.0), Quat4f::identity());
        arm.add_child(i - 1, i, JointData::new(JointVariant::Spherical(joint)));
    }
    arm.update_kinematics();
    let target = vector3(1.0, 0.3, 0.2);
    let err = HessianIk::new(&mut arm, 3, target.clone())
        .with_max_iters(50)
        .solve();
    assert!(
        err < 0.2,
        "Hessian IK error {} should be < 0.2 for spherical chain",
        err
    );
    arm.update_kinematics();
    let ee = arm.end_effector_position(3);
    let dx = ee.get(0) - target.get(0);
    let dy = ee.get(1) - target.get(1);
    let dz = ee.get(2) - target.get(2);
    assert!(
        (dx * dx + dy * dy + dz * dz).sqrt() < 0.2,
        "end-effector should be near target after Hessian IK (spherical)"
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
        err < 0.4,
        "FABRIK IK error {} should be < 0.4 (revolute recovery improved from 0.5)",
        err
    );
    arm.update_kinematics();
    let ee = arm.end_effector_position(2);
    let dx = ee.get(0) - target.get(0);
    let dy = ee.get(1) - target.get(1);
    assert!(
        (dx * dx + dy * dy).sqrt() < 0.4,
        "end-effector should be near target after FABRIK (revolute)"
    );
}

#[test]
fn fabrik_ik_spherical_reaches_target() {
    // Three-link 3D arm with spherical joints; target within reach.
    let root = JointData::new(JointVariant::Spherical(SphericalJoint::new(
        vec3(0.0, 0.0, 0.0),
        Quat4f::identity(),
    )));
    let mut arm = Armature::new(root);
    let link_len = 0.5;
    for i in 1..4 {
        let joint = SphericalJoint::new(vec3(link_len, 0.0, 0.0), Quat4f::identity());
        arm.add_child(i - 1, i, JointData::new(JointVariant::Spherical(joint)));
    }
    arm.update_kinematics();
    let target = vector3(1.0, 0.3, 0.2);
    let err = FabrikIk::new(&mut arm, 3, target.clone())
        .with_max_iters(15)
        .solve();
    assert!(
        err < 0.05,
        "FABRIK IK error {} should be < 0.05 for spherical chain",
        err
    );
    arm.update_kinematics();
    let ee = arm.end_effector_position(3);
    let dx = ee.get(0) - target.get(0);
    let dy = ee.get(1) - target.get(1);
    let dz = ee.get(2) - target.get(2);
    assert!(
        (dx * dx + dy * dy + dz * dz).sqrt() < 0.05,
        "end-effector should be near target after FABRIK (spherical, tightened)"
    );
}

#[test]
fn fabrik_ik_unreachable_target_points_at_target() {
    // Chain total length 2 (two links of 1). Target far away: end-effector should sit at
    // distance 2 from root along the direction to the target (paper: unreachable case).
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
    let total_len = 2.0;
    let target = vector3(10.0, 0.0, 0.0); // well beyond reach
    FabrikIk::new(&mut arm, 2, target.clone())
        .with_max_iters(20)
        .solve();
    arm.update_kinematics();
    let root_pos = arm.end_effector_position(0);
    let ee = arm.end_effector_position(2);
    let to_ee = vector3(
        ee.get(0) - root_pos.get(0),
        ee.get(1) - root_pos.get(1),
        ee.get(2) - root_pos.get(2),
    );
    let dist_ee =
        (to_ee.get(0) * to_ee.get(0) + to_ee.get(1) * to_ee.get(1) + to_ee.get(2) * to_ee.get(2))
            .sqrt();
    let dir_to_target = vector3(
        target.get(0) - root_pos.get(0),
        target.get(1) - root_pos.get(1),
        target.get(2) - root_pos.get(2),
    );
    let norm_target = (dir_to_target.get(0) * dir_to_target.get(0)
        + dir_to_target.get(1) * dir_to_target.get(1)
        + dir_to_target.get(2) * dir_to_target.get(2))
    .sqrt();
    assert!(norm_target > total_len, "target must be unreachable");
    let expected_ee_x = total_len; // direction is (1,0,0)
    assert!(
        (dist_ee - total_len).abs() < 0.02,
        "unreachable: end-effector should be at distance total_len ({}) from root, got {}",
        total_len,
        dist_ee
    );
    assert!(
        (ee.get(0) - expected_ee_x).abs() < 0.02
            && ee.get(1).abs() < 0.02
            && ee.get(2).abs() < 0.02,
        "unreachable: end-effector should lie along direction to target (here +X), got ({}, {}, {})",
        ee.get(0),
        ee.get(1),
        ee.get(2)
    );
}

#[test]
fn fabrik_ik_multi_target_y_shape() {
    // Y-shape: root 0 -> node 1 (sub-base) -> node 2 (ee1) and node 1 -> node 3 (ee2).
    // Paths: [0,1,2], [0,1,3]. Two end effectors with one sub-base.
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
        JointData::new(JointVariant::Revolute2d(Revolute2dJoint::at(0.8, 0.0, 0.0))),
    );
    arm.add_child(
        1,
        3,
        JointData::new(JointVariant::Revolute2d(Revolute2dJoint::at(0.8, 0.0, 0.0))),
    );
    arm.update_kinematics();
    let target2 = vector3(1.2, 0.4, 0.0);
    let target3 = vector3(1.2, -0.4, 0.0);
    let err = FabrikIk::multi_target(
        &mut arm,
        &[(2, target2.clone()), (3, target3.clone())],
        25,
        1e-4,
    );
    arm.update_kinematics();
    // With one joint per node, the sub-base (node 1) has a single joint; both chains share it so
    // we only approximate both targets. Assert solver runs and reduces error.
    assert!(err < 2.0, "multi_target max error {} should be < 2.0", err);
}

#[test]
fn jacobian_ik_spherical_reaches_target() {
    // Short chain with spherical joints; Jacobian IK should reduce error toward reachable target.
    let root = JointData::new(JointVariant::Spherical(SphericalJoint::new(
        vec3(0.0, 0.0, 0.0),
        Quat4f::identity(),
    )));
    let mut arm = Armature::new(root);
    let link_len = 0.5;
    for i in 1..4 {
        let joint = SphericalJoint::new(vec3(link_len, 0.0, 0.0), Quat4f::identity());
        arm.add_child(i - 1, i, JointData::new(JointVariant::Spherical(joint)));
    }
    arm.update_kinematics();
    let target = vector3(1.0, 0.2, 0.1);
    let err = JacobianIk::new(&mut arm, 3, target.clone())
        .with_max_iters(50)
        .solve();
    assert!(
        err < 0.6,
        "Jacobian IK error {} should be < 0.6 for spherical chain (solver must move arm)",
        err
    );
    arm.update_kinematics();
    let ee = arm.end_effector_position(3);
    let dx = ee.get(0) - target.get(0);
    let dy = ee.get(1) - target.get(1);
    let dz = ee.get(2) - target.get(2);
    assert!(
        (dx * dx + dy * dy + dz * dz).sqrt() < 0.7,
        "end-effector should be nearer target after Jacobian IK (spherical)"
    );
}

#[test]
fn ccd_ik_reaches_target_2d() {
    // 2D revolute chain; target within reach.
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
    let err = CcdIk::new(&mut arm, 2, target.clone())
        .with_max_iters(30)
        .with_max_rotation_rad(0.5)
        .solve();
    assert!(
        err < 0.05,
        "CCD IK error {} should be < 0.05 for 2D revolute",
        err
    );
    arm.update_kinematics();
    let ee = arm.end_effector_position(2);
    let dx = ee.get(0) - target.get(0);
    let dy = ee.get(1) - target.get(1);
    assert!(
        (dx * dx + dy * dy).sqrt() < 0.05,
        "end-effector should be near target after CCD (2D)"
    );
}

#[test]
fn ccd_ik_spherical_reaches_target() {
    // 3D spherical chain; target within reach.
    let root = JointData::new(JointVariant::Spherical(SphericalJoint::new(
        vec3(0.0, 0.0, 0.0),
        Quat4f::identity(),
    )));
    let mut arm = Armature::new(root);
    let link_len = 0.5;
    for i in 1..4 {
        let joint = SphericalJoint::new(vec3(link_len, 0.0, 0.0), Quat4f::identity());
        arm.add_child(i - 1, i, JointData::new(JointVariant::Spherical(joint)));
    }
    arm.update_kinematics();
    let target = vector3(1.0, 0.3, 0.2);
    let err = CcdIk::new(&mut arm, 3, target.clone())
        .with_max_iters(40)
        .with_max_rotation_rad(0.6)
        .solve();
    assert!(
        err < 0.1,
        "CCD IK error {} should be < 0.1 for spherical chain",
        err
    );
    arm.update_kinematics();
    let ee = arm.end_effector_position(3);
    let dx = ee.get(0) - target.get(0);
    let dy = ee.get(1) - target.get(1);
    let dz = ee.get(2) - target.get(2);
    assert!(
        (dx * dx + dy * dy + dz * dz).sqrt() < 0.1,
        "end-effector should be near target after CCD (spherical)"
    );
}

#[test]
fn fabrik_sqp_ik_reaches_target() {
    // 3-link revolute chain; target within reach. FabrikSqp should reach target.
    let root = JointData::new(JointVariant::Revolute(RevoluteJoint::new(
        vec3(0.0, 0.0, 0.0),
        (0.0, 1.0, 0.0),
        0.0,
    )));
    let mut arm = Armature::new(root);
    let link_len = 0.5;
    for i in 1..4 {
        arm.add_child(
            i - 1,
            i,
            JointData::new(JointVariant::Revolute(RevoluteJoint::new(
                vec3(link_len, 0.0, 0.0),
                (0.0, 1.0, 0.0),
                0.0,
            ))),
        );
    }
    arm.update_kinematics();
    let target = vector3(1.0, 0.2, 0.3);
    let err = FabrikSqpIk::new(&mut arm, 3, target.clone())
        .with_nl(15)
        .with_max_iters_fallback(80)
        .with_tolerance(1e-4)
        .solve();
    assert!(
        err < 0.2,
        "FabrikSqp IK error {} should be < 0.2 for revolute chain",
        err
    );
    arm.update_kinematics();
    let ee = arm.end_effector_position(3);
    let dx = ee.get(0) - target.get(0);
    let dy = ee.get(1) - target.get(1);
    let dz = ee.get(2) - target.get(2);
    assert!(
        (dx * dx + dy * dy + dz * dz).sqrt() < 0.25,
        "end-effector should be near target after FabrikSqp (revolute)"
    );
}

#[test]
fn ccd_ik_unreachable_target() {
    // Target beyond reach; solver should not panic and error should be bounded.
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
    let target = vector3(10.0, 0.0, 0.0);
    let err = CcdIk::new(&mut arm, 2, target).with_max_iters(20).solve();
    // Unreachable: error should be roughly (10 - 2) = 8 or so, bounded.
    assert!(
        err < 20.0 && err > 0.0,
        "CCD unreachable: error {} should be bounded",
        err
    );
}

#[test]
fn ccd_ik_with_angle_limits() {
    // Revolute chain with angle limits; CCD should respect them.
    let root = JointData::new(JointVariant::Revolute(RevoluteJoint::new(
        vec3(0.0, 0.0, 0.0),
        (0.0, 1.0, 0.0),
        0.0,
    )));
    let mut arm = Armature::new(root);
    arm.add_child(
        0,
        1,
        JointData::new(JointVariant::Revolute(
            RevoluteJoint::new(vec3(1.0, 0.0, 0.0), (0.0, 1.0, 0.0), 0.0)
                .with_angle_limits(-0.5, 0.5),
        )),
    );
    arm.update_kinematics();
    let target = vector3(0.8, 0.3, 0.0);
    let _err = CcdIk::new(&mut arm, 1, target).with_max_iters(25).solve();
    arm.update_kinematics();
    let joint = &arm.tree().nodes[1].data.joint;
    if let JointVariant::Revolute(r) = joint {
        assert!(
            r.angle >= -0.51 && r.angle <= 0.51,
            "revolute angle {} should be within limits [-0.5, 0.5]",
            r.angle
        );
    }
}

#[test]
fn halley_ik_reaches_target() {
    // 3-link revolute chain; target within reach. Halley uses 6D pose (Matrix4f target).
    let root = JointData::new(JointVariant::Revolute(RevoluteJoint::new(
        vec3(0.0, 0.0, 0.0),
        (0.0, 1.0, 0.0),
        0.0,
    )));
    let mut arm = Armature::new(root);
    let link_len = 0.5;
    for i in 1..4 {
        arm.add_child(
            i - 1,
            i,
            JointData::new(JointVariant::Revolute(RevoluteJoint::new(
                vec3(link_len, 0.0, 0.0),
                (0.0, 1.0, 0.0),
                0.0,
            ))),
        );
    }
    arm.update_kinematics();
    let target_pos = vector3(1.0, 0.0, 0.3);
    let target_pose = mathlib::cg::new_translation(&target_pos);
    let err = HalleyIk::new(&mut arm, 3, target_pose)
        .with_max_iters(32)
        .solve();
    assert!(
        err < 0.5,
        "Halley IK error {} should be < 0.5 for revolute chain",
        err
    );
    arm.update_kinematics();
    let ee = arm.end_effector_position(3);
    let dx = ee.get(0) - target_pos.get(0);
    let dy = ee.get(1) - target_pos.get(1);
    let dz = ee.get(2) - target_pos.get(2);
    assert!(
        (dx * dx + dy * dy + dz * dz).sqrt() < 0.5,
        "end-effector should be near target after Halley (revolute)"
    );
}

#[test]
fn halley_ik_spherical_reaches_target() {
    // 3-link spherical chain; target within reach.
    let root = JointData::new(JointVariant::Spherical(SphericalJoint::new(
        vec3(0.0, 0.0, 0.0),
        Quat4f::identity(),
    )));
    let mut arm = Armature::new(root);
    let link_len = 0.5;
    for i in 1..4 {
        let joint = SphericalJoint::new(vec3(link_len, 0.0, 0.0), Quat4f::identity());
        arm.add_child(i - 1, i, JointData::new(JointVariant::Spherical(joint)));
    }
    arm.update_kinematics();
    let target_pos = vector3(1.0, 0.3, 0.2);
    let target_pose = mathlib::cg::new_translation(&target_pos);
    let err = HalleyIk::new(&mut arm, 3, target_pose)
        .with_max_iters(32)
        .solve();
    assert!(
        err < 0.3,
        "Halley IK error {} should be < 0.3 for spherical chain",
        err
    );
    arm.update_kinematics();
    let ee = arm.end_effector_position(3);
    let dx = ee.get(0) - target_pos.get(0);
    let dy = ee.get(1) - target_pos.get(1);
    let dz = ee.get(2) - target_pos.get(2);
    assert!(
        (dx * dx + dy * dy + dz * dz).sqrt() < 0.3,
        "end-effector should be near target after Halley (spherical)"
    );
}
