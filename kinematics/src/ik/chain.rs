//! Shared helpers for building Jacobians/Hessians along a serial chain.

use mathlib::Vector;
use mathlib::math3d::Vector3f;
use mathlib::{Matrix, Matrix4f, Storage, transform_vector, vec3_cross_f64, vector3_cross};

use crate::armature::{Armature, JointData, JointVariant};

/// Applies a joint-space step to theta using path-to-DFS mapping and world-to-local
/// conversion for spherical joints. Shared by JacobianIk, HessianIk, and HalleyIk.
#[allow(clippy::module_name_repetitions)]
pub fn apply_joint_step(
    tree: &mathlib::graph::Tree<JointData>,
    path: &[usize],
    dof_mapping: &[usize],
    alpha: f32,
    d_theta: &[f32],
    theta_new: &mut [f32],
) {
    let mut path_dof_offset = 0;
    for (path_node_idx, &node_idx) in path.iter().enumerate() {
        let dof = tree.nodes[node_idx].data.joint.dof_count();
        if dof == 0 {
            continue;
        }
        let dfs_start = dof_mapping[path_dof_offset];
        if dof == 3 && path_node_idx > 0 {
            let parent_idx = path[path_node_idx - 1];
            let parent_world = &tree.nodes[parent_idx].data.world_transform;
            let parent_r_t = parent_world.transpose();
            let mut d_world = Vector3f::with_capacity(3);
            d_world.set(0, d_theta[path_dof_offset]);
            d_world.set(1, d_theta[path_dof_offset + 1]);
            d_world.set(2, d_theta[path_dof_offset + 2]);
            let d_local = transform_vector(&parent_r_t, &d_world);
            for d in 0..3 {
                if dfs_start + d < theta_new.len() {
                    theta_new[dfs_start + d] += alpha * d_local.get(d);
                }
            }
        } else {
            for d in 0..dof {
                if dfs_start + d < theta_new.len() && path_dof_offset + d < d_theta.len() {
                    theta_new[dfs_start + d] += alpha * d_theta[path_dof_offset + d];
                }
            }
        }
        path_dof_offset += dof;
    }
}

/// World-space origin extracted from a transform matrix.
#[must_use]
pub fn world_position(transform: &Matrix4f) -> Vector3f {
    let mut out = Vector3f::with_capacity(3);
    out.set(0, transform.get(0, 3));
    out.set(1, transform.get(1, 3));
    out.set(2, transform.get(2, 3));
    out
}

/// World-space direction (ignores translation) for a local axis.
#[must_use]
pub fn world_direction(transform: &Matrix4f, axis: &Vector3f) -> Vector3f {
    transform_vector(transform, axis)
}

/// Builds the 6×N geometric Jacobian (linear rows first, angular rows second) and a
/// parallel vector indicating whether each DOF is prismatic.
#[must_use]
pub fn build_geometric_jacobian(
    armature: &Armature,
    path: &[usize],
    end_effector_pos: &Vector3f,
) -> (Matrix<f64>, Vec<bool>) {
    let dof_total: usize = path
        .iter()
        .map(|&i| armature.tree().nodes[i].data.joint.dof_count())
        .sum();
    let mut jac = Matrix::with_storage(6, dof_total, Storage::Column);
    jac.set_zero();
    let mut is_prismatic = Vec::with_capacity(dof_total);
    let mut col = 0;

    for &idx in path {
        let node = &armature.tree().nodes[idx];
        let joint_pos = world_position(&node.data.world_transform);
        let r = diff(end_effector_pos, &joint_pos);
        match &node.data.joint {
            JointVariant::Revolute(rev) => {
                let axis_local = vector_from_tuple(rev.axis);
                let axis_world = world_direction(&node.data.world_transform, &axis_local);
                let col_vec = vector3_cross(&axis_world, &r);
                set_column(&mut jac, col, &col_vec, &axis_world);
                is_prismatic.push(false);
                col += 1;
            }
            JointVariant::Prismatic(prism) => {
                let axis_local = vector_from_tuple(prism.axis);
                let axis_world = world_direction(&node.data.world_transform, &axis_local);
                set_prismatic_column(&mut jac, col, &axis_world);
                is_prismatic.push(true);
                col += 1;
            }
            JointVariant::Spherical(_) => {
                for &(ax, ay, az) in &[(1.0, 0.0, 0.0), (0.0, 1.0, 0.0), (0.0, 0.0, 1.0)] {
                    let axis_local = vector3(ax, ay, az);
                    let axis_world = world_direction(&node.data.world_transform, &axis_local);
                    let col_vec = vector3_cross(&axis_world, &r);
                    set_column(&mut jac, col, &col_vec, &axis_world);
                    is_prismatic.push(false);
                    col += 1;
                }
            }
            JointVariant::Revolute2d(_) => {
                let axis = vector3(0.0, 0.0, 1.0);
                let axis_world = world_direction(&node.data.world_transform, &axis);
                let col_vec = vector3_cross(&axis_world, &r);
                set_column(&mut jac, col, &col_vec, &axis_world);
                is_prismatic.push(false);
                col += 1;
            }
            JointVariant::Prismatic2d(prism) => {
                let axis = vector3(prism.axis.0, prism.axis.1, 0.0);
                let axis_world = world_direction(&node.data.world_transform, &axis);
                set_prismatic_column(&mut jac, col, &axis_world);
                is_prismatic.push(true);
                col += 1;
            }
            JointVariant::Fixed(_) | JointVariant::Fixed2d(_) => {}
        }
    }

    (jac, is_prismatic)
}

/// Builds the 3×N position Jacobian, world-space axes per DOF (for Hessian K term), and
/// whether each DOF is prismatic. Used by HessianIk (Erleben & Andrews, MIG 2017).
///
/// - J_col = axis_world × (ee_pos - joint_pos) for revolute/spherical; axis_world for prismatic.
/// - axes[j] = world-space axis for DOF j (zero for prismatic when computing K; stored for consistency).
#[must_use]
pub fn build_position_jacobian_and_axes(
    armature: &Armature,
    path: &[usize],
    end_effector_pos: &Vector3f,
) -> (Matrix<f64>, Vec<[f64; 3]>, Vec<bool>) {
    let dof_total: usize = path
        .iter()
        .map(|&i| armature.tree().nodes[i].data.joint.dof_count())
        .sum();
    let mut jac = Matrix::with_storage(3, dof_total, Storage::Column);
    jac.set_zero();
    let mut axes = Vec::with_capacity(dof_total);
    let mut is_prismatic = Vec::with_capacity(dof_total);
    let mut col = 0;

    for &idx in path {
        let node = &armature.tree().nodes[idx];
        let joint_pos = world_position(&node.data.world_transform);
        let r = diff(end_effector_pos, &joint_pos);
        match &node.data.joint {
            JointVariant::Revolute(rev) => {
                let axis_local = vector_from_tuple(rev.axis);
                let axis_world = world_direction(&node.data.world_transform, &axis_local);
                let col_vec = vector3_cross(&axis_world, &r);
                set_position_column(&mut jac, col, &col_vec);
                axes.push([
                    axis_world.get(0) as f64,
                    axis_world.get(1) as f64,
                    axis_world.get(2) as f64,
                ]);
                is_prismatic.push(false);
                col += 1;
            }
            JointVariant::Prismatic(prism) => {
                let axis_local = vector_from_tuple(prism.axis);
                let axis_world = world_direction(&node.data.world_transform, &axis_local);
                set_position_column(&mut jac, col, &axis_world);
                axes.push([0.0, 0.0, 0.0]);
                is_prismatic.push(true);
                col += 1;
            }
            JointVariant::Spherical(_) => {
                for &(ax, ay, az) in &[(1.0, 0.0, 0.0), (0.0, 1.0, 0.0), (0.0, 0.0, 1.0)] {
                    let axis_local = vector3(ax, ay, az);
                    let axis_world = world_direction(&node.data.world_transform, &axis_local);
                    let col_vec = vector3_cross(&axis_world, &r);
                    set_position_column(&mut jac, col, &col_vec);
                    axes.push([
                        axis_world.get(0) as f64,
                        axis_world.get(1) as f64,
                        axis_world.get(2) as f64,
                    ]);
                    is_prismatic.push(false);
                    col += 1;
                }
            }
            JointVariant::Revolute2d(_) => {
                let axis = vector3(0.0, 0.0, 1.0);
                let axis_world = world_direction(&node.data.world_transform, &axis);
                let col_vec = vector3_cross(&axis_world, &r);
                set_position_column(&mut jac, col, &col_vec);
                axes.push([
                    axis_world.get(0) as f64,
                    axis_world.get(1) as f64,
                    axis_world.get(2) as f64,
                ]);
                is_prismatic.push(false);
                col += 1;
            }
            JointVariant::Prismatic2d(prism) => {
                let axis = vector3(prism.axis.0, prism.axis.1, 0.0);
                let axis_world = world_direction(&node.data.world_transform, &axis);
                set_position_column(&mut jac, col, &axis_world);
                axes.push([0.0, 0.0, 0.0]);
                is_prismatic.push(true);
                col += 1;
            }
            JointVariant::Fixed(_) | JointVariant::Fixed2d(_) => {}
        }
    }

    (jac, axes, is_prismatic)
}

fn set_position_column(jac: &mut Matrix<f64>, col: usize, v: &Vector3f) {
    for row in 0..3 {
        jac.set(row, col, v.get(row) as f64);
    }
}

fn diff(a: &Vector3f, b: &Vector3f) -> Vector3f {
    let mut out = Vector3f::with_capacity(3);
    out.set(0, a.get(0) - b.get(0));
    out.set(1, a.get(1) - b.get(1));
    out.set(2, a.get(2) - b.get(2));
    out
}

fn vector_from_tuple(axis: (f32, f32, f32)) -> Vector3f {
    vector3(axis.0, axis.1, axis.2)
}

fn vector3(x: f32, y: f32, z: f32) -> Vector3f {
    let mut v = Vector3f::with_capacity(3);
    v.set(0, x);
    v.set(1, y);
    v.set(2, z);
    v
}

fn set_column(jac: &mut Matrix<f64>, col: usize, linear: &Vector3f, angular: &Vector3f) {
    for row in 0..3 {
        jac.set(row, col, linear.get(row) as f64);
        jac.set(row + 3, col, angular.get(row) as f64);
    }
}

fn set_prismatic_column(jac: &mut Matrix<f64>, col: usize, axis_world: &Vector3f) {
    for row in 0..3 {
        jac.set(row, col, axis_world.get(row) as f64);
        jac.set(row + 3, col, 0.0);
    }
}

/// Adds the Hessian product `H * step` into `out`, starting from `jacobian`.
pub fn hessian_product(
    jacobian: &Matrix<f64>,
    step: &Vector<f64>,
    is_prismatic: &[bool],
    out: &mut Matrix<f64>,
) {
    out.copy_from(jacobian);
    let dof = jacobian.cols();
    assert_eq!(dof, step.rows());
    assert_eq!(is_prismatic.len(), dof);

    for k in 0..dof {
        let jvk = column_segment(jacobian, k, 0);
        let jwk = column_segment(jacobian, k, 3);

        for i in 0..k {
            if !is_prismatic[i] {
                let jwi = column_segment(jacobian, i, 3);
                if !is_prismatic[k] {
                    let rot = vec3_cross_f64(&jwi, &jwk);
                    add_segment(out, k, 3, &(scale(&rot, step.get(i))));
                }
                let trans = vec3_cross_f64(&jwi, &jvk);
                add_segment(out, k, 0, &(scale(&trans, step.get(i))));
                add_segment(out, i, 0, &(scale(&trans, step.get(k))));
            }
        }

        if !is_prismatic[k] {
            let diag = vec3_cross_f64(&jwk, &jvk);
            add_segment(out, k, 0, &(scale(&diag, step.get(k))));
        }
    }
}

fn column_segment(matrix: &Matrix<f64>, col: usize, row_offset: usize) -> [f64; 3] {
    [
        matrix.get(row_offset, col),
        matrix.get(row_offset + 1, col),
        matrix.get(row_offset + 2, col),
    ]
}

fn add_segment(matrix: &mut Matrix<f64>, col: usize, row_offset: usize, value: &[f64; 3]) {
    for (r, &v) in value.iter().enumerate() {
        let current = matrix.get(row_offset + r, col);
        matrix.set(row_offset + r, col, current + v);
    }
}

fn scale(v: &[f64; 3], s: f64) -> [f64; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}
