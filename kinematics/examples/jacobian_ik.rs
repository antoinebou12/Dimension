//! Jacobian IK example: solve for joint angles to reach a target position.

use kinematics::ik::JacobianIk;
use kinematics::joints::RevoluteJoint;
use kinematics::{Armature, JointData, JointVariant};
use mathlib::cg::vector3;

fn main() {
    let root = JointData::new(JointVariant::Revolute(RevoluteJoint::new(
        vector3(0.0, 0.0, 0.0),
        (0.0, 1.0, 0.0),
        0.0,
    )));
    let mut arm = Armature::new(root);
    for i in 1..4 {
        arm.add_child(
            i - 1,
            i,
            JointData::new(JointVariant::Revolute(RevoluteJoint::new(
                vector3(1.0, 0.0, 0.0),
                (0.0, 1.0, 0.0),
                0.0,
            ))),
        );
    }
    arm.tree_mut().nodes[3].data.is_end_effector = true;
    arm.update_kinematics();

    let target = vector3(2.5, 0.5, 0.0);
    let err_before = {
        let ee = arm.end_effector_position(3);
        ((ee.get(0) - target.get(0)).powi(2)
            + (ee.get(1) - target.get(1)).powi(2)
            + (ee.get(2) - target.get(2)).powi(2))
        .sqrt()
    };
    println!("Error before IK: {err_before:.6}");

    let err_after = JacobianIk::new(&mut arm, 3, target)
        .with_max_iters(30)
        .solve();
    println!("Error after IK:  {err_after:.6}");

    let ee = arm.end_effector_position(3);
    println!(
        "End-effector: ({}, {}, {})",
        ee.get(0),
        ee.get(1),
        ee.get(2)
    );
}
