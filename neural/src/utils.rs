//! Utilities for normalizing positions and joint angles for neural IK.

use crate::ChainConfig;

/// Joint limits for a single DOF (min, max) in radians or meters.
pub type JointLimits = Option<(f32, f32)>;

/// Normalize target position from workspace to approximately [-1, 1].
#[must_use]
pub fn normalize_position(pos: [f32; 3], config: &ChainConfig) -> [f32; 3] {
    let [mx, my, mz] = config.workspace_min;
    let [max_x, max_y, max_z] = config.workspace_max;
    let sx = (max_x - mx).max(1e-6);
    let sy = (max_y - my).max(1e-6);
    let sz = (max_z - mz).max(1e-6);
    [
        (pos[0] - mx) / sx * 2.0 - 1.0,
        (pos[1] - my) / sy * 2.0 - 1.0,
        (pos[2] - mz) / sz * 2.0 - 1.0,
    ]
}

/// Denormalize position from [-1, 1] back to workspace.
#[must_use]
pub fn denormalize_position(norm: [f32; 3], config: &ChainConfig) -> [f32; 3] {
    let [mx, my, mz] = config.workspace_min;
    let [max_x, max_y, max_z] = config.workspace_max;
    let sx = (max_x - mx).max(1e-6);
    let sy = (max_y - my).max(1e-6);
    let sz = (max_z - mz).max(1e-6);
    [
        (norm[0] * 0.5 + 0.5) * sx + mx,
        (norm[1] * 0.5 + 0.5) * sy + my,
        (norm[2] * 0.5 + 0.5) * sz + mz,
    ]
}

/// Normalize joint angles to approximately [-1, 1] using limits or ±π.
pub fn normalize_joints(joints: &[f32], config: &ChainConfig, out: &mut [f32]) {
    for (i, &v) in joints.iter().enumerate().take(config.dof).take(out.len()) {
        let (lo, hi) = config
            .joint_limits
            .get(i)
            .and_then(|l| *l)
            .unwrap_or((-std::f32::consts::PI, std::f32::consts::PI));
        let span = (hi - lo).max(1e-6);
        out[i] = (v - lo) / span * 2.0 - 1.0;
    }
}

/// Denormalize joint angles from [-1, 1] back to radians (or meters).
pub fn denormalize_joints(norm: &[f32], config: &ChainConfig, out: &mut [f32]) {
    for (i, &n) in norm.iter().enumerate().take(config.dof).take(out.len()) {
        let (lo, hi) = config
            .joint_limits
            .get(i)
            .and_then(|l| *l)
            .unwrap_or((-std::f32::consts::PI, std::f32::consts::PI));
        let span = (hi - lo).max(1e-6);
        out[i] = (n * 0.5 + 0.5) * span + lo;
    }
}
