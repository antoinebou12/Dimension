//! CCD (Cyclic Coordinate Descent) IK solver.
//!
//! A classic iterative solver that processes joints from the tip toward the root. For each
//! joint, it rotates to minimize the distance from the end-effector to the target. Simple,
//! fast, and robust for real-time use.
//!
//! Supports revolute (2D/3D), spherical, and mixed chains. Fixed and prismatic joints are
//! skipped. Optional per-step rotation clamping prevents large jumps.

use mathlib::Quat4f;
use mathlib::cg::vector3;
use mathlib::math3d::{Vector3f, matrix4f_inverse, transform_vector, vector3_cross};

use super::util;
use crate::armature::{Armature, JointVariant};

/// Epsilon for length checks.
const EPS: f32 = 1e-10;

/// Default position tolerance for convergence.
const DEFAULT_TOLERANCE: f32 = 1e-5;

/// CCD IK solver for position-only chains.
pub struct CcdIk<'a> {
    armature: &'a mut Armature,
    end_effector_idx: usize,
    target: Vector3f,
    max_iters: usize,
    tolerance: f32,
    /// Maximum rotation per joint per step (radians). `PI` = no effective clamp.
    max_rotation_rad: f32,
}

impl<'a> CcdIk<'a> {
    /// Creates a CCD solver.
    pub fn new(armature: &'a mut Armature, end_effector_idx: usize, target: Vector3f) -> Self {
        Self {
            armature,
            end_effector_idx,
            target,
            max_iters: 20,
            tolerance: DEFAULT_TOLERANCE,
            max_rotation_rad: std::f32::consts::PI,
        }
    }

    /// Sets maximum iterations.
    #[must_use]
    pub fn with_max_iters(mut self, n: usize) -> Self {
        self.max_iters = n;
        self
    }

    /// Sets position tolerance for early exit.
    #[must_use]
    pub fn with_tolerance(mut self, tol: f32) -> Self {
        self.tolerance = tol;
        self
    }

    /// Sets maximum rotation per joint per step in radians.
    #[must_use]
    pub fn with_max_rotation_rad(mut self, max: f32) -> Self {
        self.max_rotation_rad = max;
        self
    }

    /// Runs CCD iterations; updates armature joint angles. Returns final error.
    pub fn solve(&mut self) -> f32 {
        let path = self.armature.path_to(self.end_effector_idx);
        if path.len() < 2 {
            return self.error();
        }

        for _ in 0..self.max_iters {
            // Iterate from the joint just before the end-effector back toward the root.
            for path_idx in (0..path.len() - 1).rev() {
                let joint_node_idx = path[path_idx];
                self.update_joint_ccd(joint_node_idx);
            }

            // Check convergence.
            let err = self.error();
            if err <= self.tolerance {
                return err;
            }
        }

        self.error()
    }

    /// Rotate joint at `joint_idx` to minimize end-effector distance to target.
    fn update_joint_ccd(&mut self, joint_idx: usize) {
        self.armature.update_kinematics();

        let (joint_kind, joint_world, axis_3d_opt) = {
            let tree = self.armature.tree();
            let joint = &tree.nodes[joint_idx].data.joint;
            let kind = match joint {
                JointVariant::Revolute(_) => JointKind::Revolute3d,
                JointVariant::Revolute2d(_) => JointKind::Revolute2d,
                JointVariant::Spherical(_) => JointKind::Spherical,
                _ => return, // Fixed, Prismatic: skip
            };
            let world = tree.nodes[joint_idx].data.world_transform.clone();
            let axis = match joint {
                JointVariant::Revolute(r) => Some((r.axis.0, r.axis.1, r.axis.2)),
                _ => None,
            };
            (kind, world, axis)
        };

        let joint_pos = self.armature.end_effector_position(joint_idx);
        let ee_pos = self.armature.end_effector_position(self.end_effector_idx);

        let to_ee = vector3(
            ee_pos.get(0) - joint_pos.get(0),
            ee_pos.get(1) - joint_pos.get(1),
            ee_pos.get(2) - joint_pos.get(2),
        );
        let to_target = vector3(
            self.target.get(0) - joint_pos.get(0),
            self.target.get(1) - joint_pos.get(1),
            self.target.get(2) - joint_pos.get(2),
        );

        let ee_norm = to_ee.norm();
        let tgt_norm = to_target.norm();
        if ee_norm < EPS || tgt_norm < EPS {
            return;
        }

        match joint_kind {
            JointKind::Revolute2d => {
                // 2D angle difference.
                let angle_ee = to_ee.get(1).atan2(to_ee.get(0));
                let angle_tgt = to_target.get(1).atan2(to_target.get(0));
                let mut delta = angle_tgt - angle_ee;
                // Normalize to [-PI, PI].
                delta = normalize_angle(delta);
                delta = delta.clamp(-self.max_rotation_rad, self.max_rotation_rad);

                let current_angle = {
                    let tree = self.armature.tree();
                    match &tree.nodes[joint_idx].data.joint {
                        JointVariant::Revolute2d(r) => r.angle,
                        _ => return,
                    }
                };
                let new_angle = current_angle + delta;
                let clamped = match self.armature.tree().nodes[joint_idx]
                    .data
                    .joint
                    .angle_limits()
                {
                    Some((lo, hi)) => new_angle.clamp(lo, hi),
                    None => new_angle,
                };
                if let JointVariant::Revolute2d(r) =
                    &mut self.armature.tree_mut().nodes[joint_idx].data.joint
                {
                    r.angle = clamped;
                }
                self.armature.update_kinematics();
            }
            JointKind::Revolute3d => {
                let (ax, ay, az) = axis_3d_opt.unwrap();
                // Transform both vectors to parent-local frame for the axis projection.
                let parent_inv = matrix4f_inverse(&joint_world);
                let ee_local = transform_vector(&parent_inv, &to_ee);
                let tgt_local = transform_vector(&parent_inv, &to_target);

                let axis = vector3(ax, ay, az);
                let ee_proj = util::project_onto_plane(&ee_local, &axis);
                let tgt_proj = util::project_onto_plane(&tgt_local, &axis);

                let ee_proj_n = ee_proj.norm();
                let tgt_proj_n = tgt_proj.norm();
                if ee_proj_n < EPS || tgt_proj_n < EPS {
                    return;
                }
                let ee_unit = vector3(
                    ee_proj.get(0) / ee_proj_n,
                    ee_proj.get(1) / ee_proj_n,
                    ee_proj.get(2) / ee_proj_n,
                );
                let tgt_unit = vector3(
                    tgt_proj.get(0) / tgt_proj_n,
                    tgt_proj.get(1) / tgt_proj_n,
                    tgt_proj.get(2) / tgt_proj_n,
                );

                let cross = vector3_cross(&ee_unit, &tgt_unit);
                let dot = ee_unit.dot(&tgt_unit);
                let sin_angle = ax * cross.get(0) + ay * cross.get(1) + az * cross.get(2);
                let mut delta = sin_angle.atan2(dot);
                delta = delta.clamp(-self.max_rotation_rad, self.max_rotation_rad);

                let current_angle = {
                    let tree = self.armature.tree();
                    match &tree.nodes[joint_idx].data.joint {
                        JointVariant::Revolute(r) => r.angle,
                        _ => return,
                    }
                };
                let new_angle = current_angle + delta;
                let clamped = match self.armature.tree().nodes[joint_idx]
                    .data
                    .joint
                    .angle_limits()
                {
                    Some((lo, hi)) => new_angle.clamp(lo, hi),
                    None => new_angle,
                };
                if let JointVariant::Revolute(r) =
                    &mut self.armature.tree_mut().nodes[joint_idx].data.joint
                {
                    r.angle = clamped;
                }
                self.armature.update_kinematics();
            }
            JointKind::Spherical => {
                // Compute full rotation quaternion from ee direction to target direction.
                let ee_dir = vector3(
                    to_ee.get(0) / ee_norm,
                    to_ee.get(1) / ee_norm,
                    to_ee.get(2) / ee_norm,
                );
                let tgt_dir = vector3(
                    to_target.get(0) / tgt_norm,
                    to_target.get(1) / tgt_norm,
                    to_target.get(2) / tgt_norm,
                );

                let q_world = util::quat_from_vec_to_vec(&ee_dir, &tgt_dir);

                // Clamp rotation angle if needed.
                let q_world = clamp_quat_angle(q_world, self.max_rotation_rad);

                // Transform rotation to parent-local frame.
                let parent_inv = matrix4f_inverse(&joint_world);
                let q_local = transform_quat_to_local(&q_world, &joint_world, &parent_inv);

                // Compose with existing quaternion.
                let existing = {
                    let tree = self.armature.tree();
                    match &tree.nodes[joint_idx].data.joint {
                        JointVariant::Spherical(s) => s.quat,
                        _ => return,
                    }
                };
                let new_quat = quat_mul(&q_local, &existing);
                if let JointVariant::Spherical(s) =
                    &mut self.armature.tree_mut().nodes[joint_idx].data.joint
                {
                    s.quat = new_quat;
                }
                self.armature.update_kinematics();
            }
        }
    }

    fn error(&mut self) -> f32 {
        self.armature.update_kinematics();
        let ee = self.armature.end_effector_position(self.end_effector_idx);
        let dx = ee.get(0) - self.target.get(0);
        let dy = ee.get(1) - self.target.get(1);
        let dz = ee.get(2) - self.target.get(2);
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}

#[derive(Clone, Copy)]
enum JointKind {
    Revolute2d,
    Revolute3d,
    Spherical,
}

/// Normalize angle to [-PI, PI].
fn normalize_angle(mut a: f32) -> f32 {
    while a > std::f32::consts::PI {
        a -= std::f32::consts::TAU;
    }
    while a < -std::f32::consts::PI {
        a += std::f32::consts::TAU;
    }
    a
}

/// Extract (axis, angle) from a unit quaternion.
fn quat_to_axis_angle(q: &Quat4f) -> (Vector3f, f32) {
    let v_norm = (q.x * q.x + q.y * q.y + q.z * q.z).sqrt();
    if v_norm < EPS {
        return (vector3(0.0, 1.0, 0.0), 0.0);
    }
    let angle = 2.0 * v_norm.atan2(q.w);
    let inv = 1.0 / v_norm;
    (vector3(q.x * inv, q.y * inv, q.z * inv), angle)
}

/// Clamp a quaternion's rotation angle to `max_rad`.
fn clamp_quat_angle(q: Quat4f, max_rad: f32) -> Quat4f {
    let (axis, angle) = quat_to_axis_angle(&q);
    if angle.abs() <= max_rad {
        q
    } else {
        let clamped = angle.clamp(-max_rad, max_rad);
        Quat4f::from_axis_angle(&axis, clamped)
    }
}

/// Transform a world-frame rotation quaternion to a parent-local rotation.
fn transform_quat_to_local(
    q_world: &Quat4f,
    _parent_world: &mathlib::Matrix4f,
    parent_inv: &mathlib::Matrix4f,
) -> Quat4f {
    let (axis, angle) = quat_to_axis_angle(q_world);
    if angle.abs() < EPS {
        return Quat4f::identity();
    }
    let axis_local = transform_vector(parent_inv, &axis);
    let n = axis_local.norm();
    if n < EPS {
        return Quat4f::identity();
    }
    Quat4f::from_axis_angle(&axis_local.normalize(), angle)
}

/// Quaternion multiplication: q1 * q2 (apply q2 first, then q1).
fn quat_mul(q1: &Quat4f, q2: &Quat4f) -> Quat4f {
    *q1 * *q2
}
