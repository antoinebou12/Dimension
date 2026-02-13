//! Forward kinematics example: build a chain and compute end-effector position.

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
                0.1 * i as f32,
            ))),
        );
    }
    arm.update_kinematics();
    let ee = arm.end_effector_position(3);
    println!(
        "End-effector position: ({}, {}, {})",
        ee.get(0),
        ee.get(1),
        ee.get(2)
    );
}
