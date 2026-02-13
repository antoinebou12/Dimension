use kinematics::{Armature, HalleyIk, JacobianIk, JointData, Revolute2dJoint};
use mathlib::Vector3f;
use mathlib::cg::matrix4f_identity;

fn build_planar_arm() -> Armature {
    use kinematics::JointVariant::Revolute2d;

    let root = JointData::new(Revolute2d(Revolute2dJoint::at(0.0, 0.0, 0.0)));
    let mut armature = Armature::new(root);
    armature.add_child(
        0,
        1,
        JointData::new(Revolute2d(Revolute2dJoint::at(1.0, 0.0, 0.0))),
    );
    armature.add_child(
        1,
        2,
        JointData::new(Revolute2d(Revolute2dJoint::at(1.0, 0.0, 0.0))).with_end_effector(true),
    );
    armature
}

fn target_transform(x: f32, y: f32, z: f32) -> mathlib::Matrix4f {
    let mut m = matrix4f_identity();
    m.set(0, 3, x);
    m.set(1, 3, y);
    m.set(2, 3, z);
    m
}

fn target_position(x: f32, y: f32, z: f32) -> Vector3f {
    let mut v = Vector3f::with_capacity(3);
    v.set(0, x);
    v.set(1, y);
    v.set(2, z);
    v
}

#[test]
fn halley_converges_to_pose() {
    let mut armature = build_planar_arm();
    let target = target_transform(1.2, 0.6, 0.0);

    let err = HalleyIk::new(&mut armature, 2, target)
        .with_max_iters(32)
        .solve();
    assert!(err < 1e-3, "Halley solver failed to reach pose (err={err})");
    armature.update_kinematics();
    let ee = armature.end_effector_position(2);
    let residual =
        ((ee.get(0) - 1.2).powi(2) + (ee.get(1) - 0.6).powi(2) + ee.get(2).powi(2)).sqrt();
    assert!(residual < 5e-2, "end effector should be near target");
}

#[test]
fn halley_beats_jacobian_on_same_goal() {
    let mut arm_halley = build_planar_arm();
    let target_tf = target_transform(0.9, 0.8, 0.0);
    let halley_err = HalleyIk::new(&mut arm_halley, 2, target_tf.clone())
        .with_max_iters(32)
        .solve();

    let mut arm_jac = build_planar_arm();
    let target_pos = target_position(0.9, 0.8, 0.0);
    let jacobian_err = JacobianIk::new(&mut arm_jac, 2, target_pos)
        .with_max_iters(32)
        .solve();

    assert!(
        halley_err < jacobian_err * 0.8,
        "Halley should outperform Jacobian (halley={halley_err}, jacobian={jacobian_err})"
    );
}
