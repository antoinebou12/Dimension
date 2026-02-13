//! FABRIK (Forward And Backward Reaching Inverse Kinematics) solver.
//!
//! Recovers joint angles from FABRIK positions for revolute-only chains (2D and 3D).

use mathlib::Vector3f;
use mathlib::math3d::vector3_cross;

use crate::armature::{Armature, JointVariant};

/// Epsilon for length and distance checks.
const EPS: f32 = 1e-10;
const EPS_SQ: f32 = 1e-20;

/// FABRIK IK solver for position-only chains.
pub struct FabrikIk<'a> {
    armature: &'a mut Armature,
    end_effector_idx: usize,
    target: Vector3f,
    max_iters: usize,
}

impl<'a> FabrikIk<'a> {
    /// Creates a FABRIK solver.
    pub fn new(armature: &'a mut Armature, end_effector_idx: usize, target: Vector3f) -> Self {
        Self {
            armature,
            end_effector_idx,
            target,
            max_iters: 10,
        }
    }

    /// Sets maximum iterations.
    #[must_use]
    pub fn with_max_iters(mut self, n: usize) -> Self {
        self.max_iters = n;
        self
    }

    /// Runs FABRIK; updates armature joint positions. Returns final error.
    pub fn solve(&mut self) -> f32 {
        let path = self.armature.path_to(self.end_effector_idx);
        if path.len() < 2 {
            return self.error();
        }

        let positions: Vec<Vector3f> = path
            .iter()
            .map(|&i| self.armature.end_effector_position(i))
            .collect();
        let lengths: Vec<f32> = (0..positions.len() - 1)
            .map(|i| (&positions[i] - &positions[i + 1]).norm())
            .collect();
        let total_len: f32 = lengths.iter().sum();

        let mut pos = positions;
        let goal = self.target.clone();
        let root = pos[0].clone();

        let dist_to_goal = (&root - &goal).norm();
        if total_len < EPS || dist_to_goal < EPS {
            return self.error();
        }

        let n = pos.len();
        for _ in 0..self.max_iters {
            pos[n - 1] = goal.clone();
            for i in (0..n - 1).rev() {
                pos[i] = Self::place_at_length(&pos[i + 1], &pos[i], lengths[i]);
            }
            pos[0] = root.clone();
            for i in 1..pos.len() {
                pos[i] = Self::place_at_length(&pos[i - 1], &pos[i], lengths[i - 1]);
            }
        }

        self.update_armature_from_positions(&path, &pos);
        self.error()
    }

    fn error(&mut self) -> f32 {
        self.armature.update_kinematics();
        let ee = self.armature.end_effector_position(self.end_effector_idx);
        (&ee - &self.target).norm()
    }

    /// Place `toward` at distance `length` from `anchor` along the segment anchor→toward.
    fn place_at_length(anchor: &Vector3f, toward: &Vector3f, length: f32) -> Vector3f {
        let diff = toward - anchor;
        let d = diff.norm();
        let scale = if d > EPS { length / d } else { 0.0 };
        anchor + &(scale * &diff)
    }

    fn update_armature_from_positions(&mut self, path: &[usize], pos: &[Vector3f]) {
        if path.len() != pos.len() || path.len() < 2 {
            return;
        }
        let n = path.len();
        for i in 1..n {
            let parent_idx = path[i - 1];
            let child_idx = path[i];
            let (trans, is_2d, axis_3d) = {
                let tree = self.armature.tree();
                let trans = match Self::joint_translation(&tree.nodes[child_idx].data.joint) {
                    Some(t) => t,
                    None => continue,
                };
                let parent_joint = &tree.nodes[parent_idx].data.joint;
                let is_2d = matches!(parent_joint, JointVariant::Revolute2d(_));
                let axis_3d = match parent_joint {
                    JointVariant::Revolute(r) => Some((r.axis.0, r.axis.1, r.axis.2)),
                    _ => None,
                };
                (trans, is_2d, axis_3d)
            };
            let parent_pos = self.armature.end_effector_position(parent_idx);
            let target_vec = &pos[i] - &parent_pos;
            let len_sq = target_vec.dot(&target_vec);
            if len_sq < EPS_SQ {
                continue;
            }
            let angle = if is_2d {
                Some(target_vec.get(1).atan2(target_vec.get(0)) - trans.get(1).atan2(trans.get(0)))
            } else if let Some((ax, ay, az)) = axis_3d {
                let cross = vector3_cross(&trans, &target_vec);
                let dot = trans.dot(&target_vec);
                let sin_angle = ax * cross.get(0) + ay * cross.get(1) + az * cross.get(2);
                Some(sin_angle.atan2(dot))
            } else {
                None
            };
            if let Some(a) = angle {
                Self::set_joint_angle(
                    &mut self.armature.tree_mut().nodes[parent_idx].data.joint,
                    a,
                );
            }
            self.armature.update_kinematics();
        }
    }

    /// Returns the link translation (child origin in parent frame) for revolute joints.
    fn joint_translation(joint: &JointVariant) -> Option<Vector3f> {
        match joint {
            JointVariant::Revolute(r) => Some(Vector3f::from_slice(r.translation.data())),
            JointVariant::Revolute2d(r) => Some(Vector3f::from_slice(r.translation.data())),
            _ => None,
        }
    }

    /// Sets the single DOF angle on a revolute joint. Clamps to joint angle limits when present.
    fn set_joint_angle(joint: &mut JointVariant, angle: f32) {
        let clamped = match joint.angle_limits() {
            Some((lo, hi)) => angle.clamp(lo, hi),
            None => angle,
        };
        match joint {
            JointVariant::Revolute(r) => r.angle = clamped,
            JointVariant::Revolute2d(r) => r.angle = clamped,
            _ => {}
        }
    }
}
