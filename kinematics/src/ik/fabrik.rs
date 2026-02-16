//! FABRIK (Forward And Backward Reaching Inverse Kinematics) solver.
//!
//! Based on: A. Aristidou and J. Lasenby, "FABRIK: A fast, iterative solver for the Inverse
//! Kinematics problem," *Graphical Models* 73(5), pp. 243–260, 2011. The algorithm uses a
//! forward and backward reaching pass (Algorithm 1 in the paper), finding each joint position
//! as a point on a line to avoid angle/matrix operations.
//!
//! **Reachable targets**: When the target is within the chain length, a full iteration is
//! forward pass (end-effector set to target, then joints moved inward) then backward pass
//! (root restored, then joints moved outward). Iteration repeats until within
//! [tolerance](FabrikIk::with_tolerance).
//!
//! **Stagnation / relative convergence**: The solver exits early when position error improvement
//! over the last iteration is below [with_stagnation_threshold](FabrikIk::with_stagnation_threshold),
//! avoiding wasted iterations when the solution has effectively converged.
//!
//! **Degenerate segments**: When a segment direction is nearly zero, the solver uses the
//! previous iteration's segment direction as a fallback so joints do not collapse.
//!
//! **Unreachable targets**: When the target is beyond total chain length, the goal is clamped
//! to the sphere of radius `total_chain_length` around the root (same effect as the paper’s
//! single-pass stretch toward the target), so the solver converges to a well-defined pose
//! instead of oscillating.
//!
//! **Joint limits**: Revolute angle limits are applied on write-back when converting FABRIK
//! positions back to joint angles. Spherical joints are currently unconstrained (no cone or
//! ellipsoid limits as in the paper’s Section 4.3); optional joint cones may be added later.
//!
//! **Retry on rollback**: If write-back worsens error, [with_retry_on_rollback](FabrikIk::with_retry_on_rollback) runs a short retry before reverting.
//!
//! Recovers joint angles/orientations from FABRIK positions for revolute (2D/3D) and spherical
//! chains. Position-only targets; spherical joints get full 3D orientation per joint.

use std::collections::HashMap;

use mathlib::cg::vector3;
use mathlib::math3d::{matrix4f_inverse, transform_vector, vector3_cross};
use mathlib::{Quat4f, Vector3f};

use super::util;
use crate::armature::{Armature, JointVariant};

/// Epsilon for length and distance checks.
const EPS: f32 = 1e-10;
const EPS_SQ: f32 = 1e-20;

/// Default position tolerance for convergence (early exit when error below this).
const DEFAULT_TOLERANCE: f32 = 1e-5;

/// Default relative improvement threshold for stagnation (break when improvement below this).
const DEFAULT_STAGNATION_THRESHOLD: f32 = 1e-4;

/// Optional cone constraint for a segment (parent → child): the direction from parent to child
/// is constrained to lie within a cone around `axis` with half-angle `half_angle_rad`.
/// Used with [`FabrikIk::with_joint_cone`] (paper Section 4.3, rotational limits).
#[derive(Clone, Debug)]
pub struct JointConeConstraint {
    /// Parent node index (path or tree).
    pub parent_idx: usize,
    /// Child node index.
    pub child_idx: usize,
    /// Cone axis (world or parent frame); will be normalized.
    pub axis: Vector3f,
    /// Half-angle in radians; segment direction must be within this angle of the axis.
    pub half_angle_rad: f32,
}

/// FABRIK IK solver for position-only chains.
pub struct FabrikIk<'a> {
    armature: &'a mut Armature,
    end_effector_idx: usize,
    target: Vector3f,
    max_iters: usize,
    /// Stop when end-effector error is below this (in position space).
    tolerance: f32,
    /// Relative improvement below this triggers early exit (stagnation).
    stagnation_threshold: f32,
    /// When write-back worsens error, retry up to once from current pose.
    retry_on_rollback: bool,
    /// Optional cone constraints per segment (spherical joints).
    joint_cones: Option<Vec<JointConeConstraint>>,
}

impl<'a> FabrikIk<'a> {
    /// Creates a FABRIK solver.
    pub fn new(armature: &'a mut Armature, end_effector_idx: usize, target: Vector3f) -> Self {
        Self {
            armature,
            end_effector_idx,
            target,
            max_iters: 20,
            tolerance: DEFAULT_TOLERANCE,
            stagnation_threshold: DEFAULT_STAGNATION_THRESHOLD,
            retry_on_rollback: true,
            joint_cones: None,
        }
    }

    /// Adds a cone constraint for the segment from `parent_idx` to `child_idx`: the segment
    /// direction is clamped to a cone around `axis` with half-angle `half_angle_rad` (radians).
    /// Spherical joints only; axis is used as provided (e.g. world or rest direction).
    #[must_use]
    pub fn with_joint_cone(
        mut self,
        parent_idx: usize,
        child_idx: usize,
        axis: Vector3f,
        half_angle_rad: f32,
    ) -> Self {
        let v = self.joint_cones.get_or_insert_with(Vec::new);
        v.push(JointConeConstraint {
            parent_idx,
            child_idx,
            axis,
            half_angle_rad,
        });
        self
    }

    /// Sets maximum iterations.
    #[must_use]
    pub fn with_max_iters(mut self, n: usize) -> Self {
        self.max_iters = n;
        self
    }

    /// Sets position tolerance for early exit; solver stops when error is below this.
    #[must_use]
    pub fn with_tolerance(mut self, tol: f32) -> Self {
        self.tolerance = tol;
        self
    }

    /// Sets relative improvement threshold for stagnation; solver stops when
    /// `(prev_err - err) / (prev_err + eps) < threshold`.
    #[must_use]
    pub fn with_stagnation_threshold(mut self, threshold: f32) -> Self {
        self.stagnation_threshold = threshold;
        self
    }

    /// When true, on rollback (write-back worsens error) retry up to once from current pose.
    #[must_use]
    pub fn with_retry_on_rollback(mut self, retry: bool) -> Self {
        self.retry_on_rollback = retry;
        self
    }

    /// Multi–end-effector FABRIK (paper Section 4.1): sub-base positions are the centroid of
    /// proposals from each chain; then inward to root and outward to each effector.
    ///
    /// # Arguments
    /// * `armature` - The armature to solve (single root, branched chains to each effector).
    /// * `targets` - Pairs `(end_effector_node_index, target_position)`.
    /// * `max_iters` - Maximum iterations.
    /// * `tolerance` - Stop when all end-effector errors are below this.
    ///
    /// # Returns
    /// Maximum end-effector position error after the last iteration.
    pub fn multi_target(
        armature: &mut Armature,
        targets: &[(usize, Vector3f)],
        max_iters: usize,
        tolerance: f32,
    ) -> f32 {
        if targets.is_empty() {
            return 0.0;
        }
        if targets.len() == 1 {
            let (ee, t) = &targets[0];
            return FabrikIk::new(armature, *ee, t.clone())
                .with_max_iters(max_iters)
                .with_tolerance(tolerance)
                .solve();
        }
        // Build paths and common prefix (root to sub-base).
        let paths: Vec<Vec<usize>> = targets
            .iter()
            .map(|&(ee, _)| armature.path_to(ee))
            .collect();
        if paths.iter().any(|p| p.len() < 2) {
            armature.update_kinematics();
            return targets
                .iter()
                .map(|&(ee, ref t)| {
                    let pos = armature.end_effector_position(ee);
                    (&pos - t).norm()
                })
                .fold(0.0f32, |a: f32, b: f32| a.max(b));
        }
        let mut common_prefix = paths[0].clone();
        for p in paths.iter().skip(1) {
            let n = common_prefix
                .iter()
                .zip(p.iter())
                .take_while(|(a, b)| a == b)
                .count();
            common_prefix.truncate(n);
        }
        if common_prefix.is_empty() {
            return 0.0;
        }
        let sub_base = *common_prefix.last().unwrap();
        let root = paths[0][0];

        // Segment lengths per path (from current armature).
        armature.update_kinematics();
        let path_lengths: Vec<Vec<f32>> = paths
            .iter()
            .map(|path| {
                let positions: Vec<Vector3f> = path
                    .iter()
                    .map(|&i| armature.end_effector_position(i))
                    .collect();
                (0..positions.len() - 1)
                    .map(|i| (&positions[i] - &positions[i + 1]).norm())
                    .collect()
            })
            .collect();

        let total_len_per_path: Vec<f32> =
            path_lengths.iter().map(|lens| lens.iter().sum()).collect();

        let root_pos = armature.end_effector_position(root);
        let mut pos: HashMap<usize, Vector3f> = paths
            .iter()
            .flat_map(|p| p.iter().copied())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .map(|i| (i, armature.end_effector_position(i)))
            .collect();

        let mut max_err = 0.0f32;
        for _ in 0..max_iters {
            // Stage 1: from each effector inward to sub-base; sub-base = centroid; then sub-base to root.
            let mut sub_base_proposals: Vec<Vector3f> = Vec::new();
            for (ei, path) in paths.iter().enumerate() {
                let target = &targets[ei].1;
                let total_len = total_len_per_path[ei];
                let root_p = pos.get(&root).cloned().unwrap_or_else(|| root_pos.clone());
                let dist = (&root_p - target).norm();
                let goal = if dist <= EPS {
                    target.clone()
                } else if dist > total_len {
                    let dir = (target - &root_p).normalize();
                    let scaled = total_len * &dir;
                    &root_p + &scaled
                } else {
                    target.clone()
                };
                pos.insert(path[path.len() - 1], goal.clone());
                let mut j = path.len() - 2;
                while path[j] != sub_base {
                    let next_pos = pos.get(&path[j + 1]).cloned().unwrap();
                    let prev_pos = pos.get(&path[j]).cloned().unwrap();
                    let new_pos =
                        Self::place_at_length(&next_pos, &prev_pos, path_lengths[ei][j], None);
                    pos.insert(path[j], new_pos);
                    if j == 0 {
                        break;
                    }
                    j -= 1;
                }
                if path[j] == sub_base {
                    let next_pos = pos.get(&path[j + 1]).cloned().unwrap();
                    let prev_pos = pos.get(&path[j]).cloned().unwrap();
                    sub_base_proposals.push(Self::place_at_length(
                        &next_pos,
                        &prev_pos,
                        path_lengths[ei][j],
                        None,
                    ));
                }
            }
            let n_prop = sub_base_proposals.len() as f32;
            if n_prop >= 1.0 {
                let sum_x: f32 = sub_base_proposals.iter().map(|p| p.get(0)).sum();
                let sum_y: f32 = sub_base_proposals.iter().map(|p| p.get(1)).sum();
                let sum_z: f32 = sub_base_proposals.iter().map(|p| p.get(2)).sum();
                let centroid = vector3(sum_x / n_prop, sum_y / n_prop, sum_z / n_prop);
                pos.insert(sub_base, centroid);
            }
            // From sub_base inward to root.
            let sb_idx = common_prefix.iter().position(|&n| n == sub_base).unwrap();
            for j in (0..sb_idx).rev() {
                let child = common_prefix[j + 1];
                let parent = common_prefix[j];
                let child_pos = pos.get(&child).cloned().unwrap();
                let parent_pos = pos.get(&parent).cloned().unwrap();
                let len = path_lengths[0][j];
                pos.insert(
                    parent,
                    Self::place_at_length(&child_pos, &parent_pos, len, None),
                );
            }

            // Stage 2: root fixed; outward to sub_base; then each chain to effector.
            pos.insert(root, root_pos.clone());
            for j in 0..sb_idx {
                let parent = common_prefix[j];
                let child = common_prefix[j + 1];
                let parent_pos = pos.get(&parent).cloned().unwrap();
                let child_pos = pos.get(&child).cloned().unwrap();
                let len = path_lengths[0][j];
                pos.insert(
                    child,
                    Self::place_at_length(&parent_pos, &child_pos, len, None),
                );
            }
            for (ei, path) in paths.iter().enumerate() {
                let k = path.iter().position(|&n| n == sub_base).unwrap();
                for j in k..path.len() - 1 {
                    let parent = path[j];
                    let child = path[j + 1];
                    let parent_pos = pos.get(&parent).cloned().unwrap();
                    let child_pos = pos.get(&child).cloned().unwrap();
                    let len = path_lengths[ei][j];
                    pos.insert(
                        child,
                        Self::place_at_length(&parent_pos, &child_pos, len, None),
                    );
                }
            }

            max_err = 0.0;
            for (ei, path) in paths.iter().enumerate() {
                let ee = path[path.len() - 1];
                let target = &targets[ei].1;
                let p = pos.get(&ee).cloned().unwrap();
                let e = (&p - target).norm();
                if e > max_err {
                    max_err = e;
                }
            }
            if max_err <= tolerance {
                break;
            }
        }

        // Update armature from pos: collect edges (parent, child) from all paths, BFS order, then update each joint.
        let mut edges: Vec<(usize, usize)> = Vec::new();
        for path in &paths {
            for i in 1..path.len() {
                edges.push((path[i - 1], path[i]));
            }
        }
        let mut depth: HashMap<usize, usize> = HashMap::new();
        depth.insert(root, 0);
        for (p, c) in &edges {
            if let Some(&d) = depth.get(p) {
                depth.insert(*c, d + 1);
            }
        }
        edges.sort_by_key(|(p, c)| (depth.get(c).copied().unwrap_or(0), *p, *c));
        for (parent_idx, child_idx) in edges {
            if let Some(desired) = pos.get(&child_idx).cloned() {
                Self::update_joint_to_desired_child_pos(armature, parent_idx, child_idx, &desired);
            }
        }
        armature.update_kinematics();
        max_err
    }

    /// Runs FABRIK; updates armature joint positions. Returns final error.
    pub fn solve(&mut self) -> f32 {
        let path = self.armature.path_to(self.end_effector_idx);
        if path.len() < 2 {
            return self.error();
        }

        self.armature.update_kinematics();
        let err_before = {
            let ee = self.armature.end_effector_position(self.end_effector_idx);
            (&ee - &self.target).norm()
        };

        let positions: Vec<Vector3f> = path
            .iter()
            .map(|&i| self.armature.end_effector_position(i))
            .collect();
        let lengths: Vec<f32> = (0..positions.len() - 1)
            .map(|i| (&positions[i] - &positions[i + 1]).norm())
            .collect();
        let total_len: f32 = lengths.iter().sum();

        if total_len < EPS {
            return err_before;
        }

        let mut pos = positions;
        let root = pos[0].clone();

        let dist_to_goal = (&root - &self.target).norm();
        let goal: Vector3f = if dist_to_goal < EPS {
            return err_before;
        } else if dist_to_goal > total_len {
            let dir = (&self.target - &root).normalize();
            &root + &(total_len * &dir)
        } else {
            self.target.clone()
        };

        let n = pos.len();
        let path_ref = &path;
        let cones = self.joint_cones.as_ref();
        let mut prev_err_sq: Option<f32> = None;
        let mut prev_dirs: Vec<Vector3f> = (0..n - 1)
            .map(|i| (&pos[i + 1] - &pos[i]).normalize())
            .collect();
        let mut broke_on_tolerance = false;
        let mut final_err_sq_on_tolerance: f32 = 0.0;

        for _ in 0..self.max_iters {
            pos[n - 1] = goal.clone();
            for i in (0..n - 1).rev() {
                let fallback = prev_dirs.get(i);
                pos[i] = Self::place_at_length(&pos[i + 1], &pos[i], lengths[i], fallback);
                if let Some(cones) = cones {
                    for c in cones {
                        if c.parent_idx == path_ref[i] && c.child_idx == path_ref[i + 1] {
                            let dir = (&pos[i + 1] - &pos[i]).normalize();
                            let new_dir =
                                Self::project_direction_onto_cone(&dir, &c.axis, c.half_angle_rad);
                            let len = lengths[i];
                            pos[i + 1] = &pos[i] + &(len * &new_dir);
                            break;
                        }
                    }
                }
            }
            pos[0] = root.clone();
            for i in 1..pos.len() {
                let fallback = prev_dirs.get(i - 1);
                pos[i] = Self::place_at_length(&pos[i - 1], &pos[i], lengths[i - 1], fallback);
                if let Some(cones) = cones {
                    for c in cones {
                        if c.parent_idx == path_ref[i - 1] && c.child_idx == path_ref[i] {
                            let dir = (&pos[i] - &pos[i - 1]).normalize();
                            let new_dir =
                                Self::project_direction_onto_cone(&dir, &c.axis, c.half_angle_rad);
                            let len = lengths[i - 1];
                            pos[i] = &pos[i - 1] + &(len * &new_dir);
                            break;
                        }
                    }
                }
            }

            for i in 0..n - 1 {
                let d = &pos[i + 1] - &pos[i];
                let dn = d.norm();
                prev_dirs[i] = if dn > EPS {
                    let mut v = Vector3f::with_capacity(3);
                    v.set(0, d.get(0) / dn);
                    v.set(1, d.get(1) / dn);
                    v.set(2, d.get(2) / dn);
                    v
                } else {
                    prev_dirs[i].clone()
                };
            }

            let err_sq = (&pos[n - 1] - &goal).dot(&(&pos[n - 1] - &goal));
            if err_sq <= self.tolerance * self.tolerance {
                broke_on_tolerance = true;
                final_err_sq_on_tolerance = err_sq;
                break;
            }
            if let Some(pe) = prev_err_sq {
                let improvement = if pe > EPS * EPS {
                    (pe - err_sq) / (pe + EPS * EPS)
                } else {
                    0.0
                };
                if improvement < self.stagnation_threshold {
                    break;
                }
            }
            prev_err_sq = Some(err_sq);
        }

        let saved_state = self.armature.pack();
        self.update_armature_from_positions(&path, &pos);
        // update_armature_from_positions already calls update_kinematics per joint; skip redundant call
        let err_after = if broke_on_tolerance {
            final_err_sq_on_tolerance.sqrt()
        } else {
            let ee = self.armature.end_effector_position(self.end_effector_idx);
            (&ee - &self.target).norm()
        };

        if err_after > err_before {
            self.armature.unpack(&saved_state);
            if self.retry_on_rollback {
                let mut retry_pos = pos.clone();
                for _ in 0..2 {
                    retry_pos[n - 1] = goal.clone();
                    for i in (0..n - 1).rev() {
                        let fallback = prev_dirs.get(i);
                        retry_pos[i] = Self::place_at_length(
                            &retry_pos[i + 1],
                            &retry_pos[i],
                            lengths[i],
                            fallback,
                        );
                    }
                    retry_pos[0] = root.clone();
                    for i in 1..retry_pos.len() {
                        let fallback = prev_dirs.get(i - 1);
                        retry_pos[i] = Self::place_at_length(
                            &retry_pos[i - 1],
                            &retry_pos[i],
                            lengths[i - 1],
                            fallback,
                        );
                    }
                }
                self.update_armature_from_positions(&path, &retry_pos);
                let err_retry = {
                    let ee = self.armature.end_effector_position(self.end_effector_idx);
                    (&ee - &self.target).norm()
                };
                if err_retry < err_before {
                    return err_retry;
                }
                self.armature.unpack(&saved_state);
                self.armature.update_kinematics();
            }
            return err_before;
        }
        err_after
    }

    fn error(&mut self) -> f32 {
        self.armature.update_kinematics();
        let ee = self.armature.end_effector_position(self.end_effector_idx);
        (&ee - &self.target).norm()
    }

    /// Place `toward` at distance `length` from `anchor` along the segment anchor→toward.
    /// When the segment is degenerate (d ≈ 0), uses `fallback_dir` if provided and non-zero
    /// to avoid collapsing segments; otherwise returns `anchor`.
    fn place_at_length(
        anchor: &Vector3f,
        toward: &Vector3f,
        length: f32,
        fallback_dir: Option<&Vector3f>,
    ) -> Vector3f {
        let diff = toward - anchor;
        let d = diff.norm();
        if d > EPS {
            let scale = length / d;
            return anchor + &(scale * &diff);
        }
        if let Some(fb) = fallback_dir {
            let dn = fb.norm();
            if dn > EPS {
                let scale = length / dn;
                return anchor + &(scale * fb);
            }
        }
        anchor.clone()
    }

    /// Projects the segment direction onto the cone (axis, half_angle_rad). If the direction
    /// is inside the cone, returns it unchanged; otherwise returns the nearest unit vector on the cone boundary.
    fn project_direction_onto_cone(
        direction: &Vector3f,
        axis: &Vector3f,
        half_angle_rad: f32,
    ) -> Vector3f {
        let a = axis.normalize();
        let d = direction.normalize();
        let cos_h = half_angle_rad.cos();
        let sin_h = half_angle_rad.sin();
        let dot = d.dot(&a);
        let angle = dot.clamp(-1.0, 1.0).acos();
        if angle <= half_angle_rad + 1e-6 {
            return d;
        }
        let perp = &d - &(dot * &a);
        let perp_norm = perp.norm();
        let perp_unit = if perp_norm < EPS {
            let mut p = vector3(1.0, 0.0, 0.0);
            if a.get(0).abs() > 0.9 {
                p = vector3(0.0, 1.0, 0.0);
            }
            let p_dot_a = p.get(0) * a.get(0) + p.get(1) * a.get(1) + p.get(2) * a.get(2);
            let p_minus = vector3(
                p.get(0) - a.get(0) * p_dot_a,
                p.get(1) - a.get(1) * p_dot_a,
                p.get(2) - a.get(2) * p_dot_a,
            );
            let pn = p_minus.norm();
            if pn < EPS {
                vector3(0.0, 0.0, 0.0)
            } else {
                vector3(
                    p_minus.get(0) / pn,
                    p_minus.get(1) / pn,
                    p_minus.get(2) / pn,
                )
            }
        } else {
            perp.normalize()
        };
        if perp_unit.get(0).abs() + perp_unit.get(1).abs() + perp_unit.get(2).abs() < EPS {
            return d;
        }
        vector3(
            a.get(0) * cos_h + perp_unit.get(0) * sin_h,
            a.get(1) * cos_h + perp_unit.get(1) * sin_h,
            a.get(2) * cos_h + perp_unit.get(2) * sin_h,
        )
    }

    /// Updates the joint at `parent_idx` so that the child's position becomes `desired_child_pos`.
    fn update_joint_to_desired_child_pos(
        armature: &mut Armature,
        parent_idx: usize,
        child_idx: usize,
        desired_child_pos: &Vector3f,
    ) {
        let (trans, is_2d, axis_3d, is_spherical, parent_world_opt) = {
            let tree = armature.tree();
            let trans = match Self::joint_translation(&tree.nodes[child_idx].data.joint) {
                Some(t) => t,
                None => return,
            };
            let parent_joint = &tree.nodes[parent_idx].data.joint;
            let is_2d = matches!(parent_joint, JointVariant::Revolute2d(_));
            let axis_3d = match parent_joint {
                JointVariant::Revolute(r) => Some((r.axis.0, r.axis.1, r.axis.2)),
                _ => None,
            };
            let is_spherical = matches!(parent_joint, JointVariant::Spherical(_));
            // Get parent world transform for both spherical and revolute 3D joints.
            let needs_world = is_spherical || axis_3d.is_some();
            let parent_world_opt =
                needs_world.then(|| tree.nodes[parent_idx].data.world_transform.clone());
            (trans, is_2d, axis_3d, is_spherical, parent_world_opt)
        };
        let parent_pos = armature.end_effector_position(parent_idx);
        let target_vec = desired_child_pos - &parent_pos;
        let len_sq = target_vec.dot(&target_vec);
        if len_sq < EPS_SQ {
            return;
        }
        if is_spherical {
            if let Some(parent_world) = parent_world_opt {
                let parent_inv = matrix4f_inverse(&parent_world);
                let desired_dir = transform_vector(&parent_inv, &target_vec);
                let desired_norm = desired_dir.norm();
                if desired_norm > EPS {
                    let desired = desired_dir.normalize();
                    let from = trans.normalize();
                    let q = util::quat_from_vec_to_vec(&from, &desired);
                    Self::set_joint_quat(&mut armature.tree_mut().nodes[parent_idx].data.joint, q);
                }
            }
        } else {
            let angle = if is_2d {
                Some(target_vec.get(1).atan2(target_vec.get(0)) - trans.get(1).atan2(trans.get(0)))
            } else if let Some((ax, ay, az)) = axis_3d {
                // Transform target_vec to parent-local frame so both vectors
                // are in the same coordinate space before computing the angle.
                let parent_world = parent_world_opt.as_ref().unwrap();
                let parent_inv = matrix4f_inverse(parent_world);
                let target_local = transform_vector(&parent_inv, &target_vec);

                // Project both onto the plane perpendicular to the joint axis.
                let axis = vector3(ax, ay, az);
                let trans_proj = util::project_onto_plane(&trans, &axis);
                let target_proj = util::project_onto_plane(&target_local, &axis);

                let trans_norm = trans_proj.norm();
                let target_norm = target_proj.norm();
                if trans_norm > EPS && target_norm > EPS {
                    let t_n = vector3(
                        trans_proj.get(0) / trans_norm,
                        trans_proj.get(1) / trans_norm,
                        trans_proj.get(2) / trans_norm,
                    );
                    let g_n = vector3(
                        target_proj.get(0) / target_norm,
                        target_proj.get(1) / target_norm,
                        target_proj.get(2) / target_norm,
                    );
                    let cross = vector3_cross(&t_n, &g_n);
                    let dot = t_n.dot(&g_n);
                    let sin_angle = ax * cross.get(0) + ay * cross.get(1) + az * cross.get(2);
                    Some(sin_angle.atan2(dot))
                } else {
                    None
                }
            } else {
                None
            };
            if let Some(a) = angle {
                Self::set_joint_angle(&mut armature.tree_mut().nodes[parent_idx].data.joint, a);
            }
        }
        armature.update_kinematics();
    }

    fn update_armature_from_positions(&mut self, path: &[usize], pos: &[Vector3f]) {
        if path.len() != pos.len() || path.len() < 2 {
            return;
        }
        let n = path.len();
        for i in 1..n {
            Self::update_joint_to_desired_child_pos(self.armature, path[i - 1], path[i], &pos[i]);
        }
    }

    /// Returns the link translation (child origin in parent frame) for revolute and spherical joints.
    fn joint_translation(joint: &JointVariant) -> Option<Vector3f> {
        match joint {
            JointVariant::Revolute(r) => Some(Vector3f::from_slice(r.translation.data())),
            JointVariant::Revolute2d(r) => Some(Vector3f::from_slice(r.translation.data())),
            JointVariant::Spherical(s) => Some(Vector3f::from_slice(s.translation.data())),
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

    /// Sets the orientation quaternion on a spherical joint.
    fn set_joint_quat(joint: &mut JointVariant, q: Quat4f) {
        if let JointVariant::Spherical(s) = joint {
            s.quat = q;
        }
    }
}
