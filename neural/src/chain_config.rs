//! Chain configuration for neural IK: DOF count, workspace bounds, joint limits.
//!
//! Describes the serial chain so the model and dataset know input/output dimensions
//! and how to normalize target positions and joint angles.

/// Configuration for a serial kinematic chain used by neural IK.
///
/// The model is **chain-dependent**: it is trained for a fixed number of joints `n`
/// and predicts `dof` joint angles from a target position (and optionally current state).
#[derive(Clone, Debug)]
pub struct ChainConfig {
    /// Number of joint DOFs (length of pack() from root to end-effector along the IK path).
    pub dof: usize,
    /// Whether the model input includes current joint state (dof extra inputs).
    pub use_current_state: bool,
    /// Workspace bounds for target position [min_x, min_y, min_z, max_x, max_y, max_z].
    /// Used to normalize target position to roughly [-1, 1] for training.
    pub workspace_min: [f32; 3],
    /// Workspace max (see workspace_min).
    pub workspace_max: [f32; 3],
    /// Per-DOF joint limits (radians or meters). None = no limit for that DOF.
    pub joint_limits: Vec<Option<(f32, f32)>>,
}

impl ChainConfig {
    /// Input size for the neural network: 3 (target xyz) + optionally dof (current state).
    #[must_use]
    pub fn input_size(&self) -> usize {
        let base = 3;
        if self.use_current_state {
            base + self.dof
        } else {
            base
        }
    }

    /// Output size (joint angles / DOF).
    #[must_use]
    pub fn output_size(&self) -> usize {
        self.dof
    }

    /// Default workspace: cube from -2 to 2 on each axis.
    #[must_use]
    pub fn default_workspace() -> ([f32; 3], [f32; 3]) {
        ([-2.0, -2.0, -2.0], [2.0, 2.0, 2.0])
    }

    /// Creates a chain config with no joint limits.
    #[must_use]
    pub fn new(dof: usize, use_current_state: bool) -> Self {
        let (wmin, wmax) = Self::default_workspace();
        Self {
            dof,
            use_current_state,
            workspace_min: wmin,
            workspace_max: wmax,
            joint_limits: (0..dof).map(|_| None).collect(),
        }
    }

    /// Sets workspace bounds (for normalization).
    #[must_use]
    pub fn with_workspace(mut self, min: [f32; 3], max: [f32; 3]) -> Self {
        self.workspace_min = min;
        self.workspace_max = max;
        self
    }

    /// Sets joint limits per DOF (min, max) in radians.
    #[must_use]
    pub fn with_joint_limits(mut self, limits: Vec<Option<(f32, f32)>>) -> Self {
        if limits.len() >= self.dof {
            self.joint_limits = limits.into_iter().take(self.dof).collect();
        }
        self
    }
}
