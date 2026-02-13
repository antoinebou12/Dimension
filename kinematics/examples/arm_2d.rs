//! 2D arm example: two Revolute2d joints, forward kinematics, print end-effector.

use kinematics::joints::Revolute2dJoint;
use kinematics::{Armature, JointData, JointVariant};
use std::f32::consts::FRAC_PI_4;

fn main() {
    let root = JointData::new(JointVariant::Revolute2d(Revolute2dJoint::at(0.0, 0.0, 0.0)));
    let mut arm = Armature::new(root);
    arm.add_child(
        0,
        1,
        JointData::new(JointVariant::Revolute2d(Revolute2dJoint::at(
            1.0, 0.0, FRAC_PI_4,
        ))),
    );
    arm.update_kinematics();
    let ee = arm.end_effector_position(1);
    println!(
        "End-effector position: ({}, {}, {})",
        ee.get(0),
        ee.get(1),
        ee.get(2)
    );
}
