//! Revolute joint: 1 DOF rotation about an axis.

use super::Joint;
use mathlib::cg::new_translation;
use mathlib::{Matrix4f, Vector3f};

/// Revolute joint: 1 DOF rotation about axis (default Y).
#[derive(Clone, Debug)]
pub struct RevoluteJoint {
    /// Local translation offset.
    pub translation: Vector3f,
    /// Axis of rotation (x, y, z), unit vector.
    pub axis: (f32, f32, f32),
    /// Angle in radians.
    pub angle: f32,
    /// Optional minimum angle (radians). When set, [`Self::angle`] is clamped to `[angle_min, angle_max]` in [`Self::local_transform`] and [`Self::unpack`].
    pub angle_min: Option<f32>,
    /// Optional maximum angle (radians). When set with [`Self::angle_min`], angle is clamped.
    pub angle_max: Option<f32>,
}

impl Default for RevoluteJoint {
    fn default() -> Self {
        let mut t = Vector3f::with_capacity(3);
        t.set_zero();
        Self {
            translation: t,
            axis: (0.0, 1.0, 0.0),
            angle: 0.0,
            angle_min: None,
            angle_max: None,
        }
    }
}

impl RevoluteJoint {
    /// Creates a revolute joint. Axis defaults to Y.
    #[must_use]
    pub fn new(translation: Vector3f, axis: (f32, f32, f32), angle: f32) -> Self {
        Self {
            translation,
            axis,
            angle,
            angle_min: None,
            angle_max: None,
        }
    }

    /// Sets optional angle limits (radians). Returns `self` for chaining.
    #[must_use]
    pub fn with_angle_limits(mut self, min: f32, max: f32) -> Self {
        self.angle_min = Some(min);
        self.angle_max = Some(max);
        self
    }

    /// Angle clamped to [`Self::angle_min`] and [`Self::angle_max`] when present.
    fn clamped_angle(&self) -> f32 {
        match (self.angle_min, self.angle_max) {
            (Some(lo), Some(hi)) => self.angle.clamp(lo, hi),
            _ => self.angle,
        }
    }

    fn rotation_matrix(&self) -> mathlib::Matrix4f {
        let a = self.clamped_angle();
        let mut axis_angle = Vector3f::with_capacity(3);
        axis_angle.set(0, self.axis.0 * a);
        axis_angle.set(1, self.axis.1 * a);
        axis_angle.set(2, self.axis.2 * a);
        mathlib::cg::from_scaled_axis(&axis_angle)
    }
}

impl Joint for RevoluteJoint {
    fn local_transform(&self) -> Matrix4f {
        debug_assert!(
            (self.axis.0 * self.axis.0 + self.axis.1 * self.axis.1 + self.axis.2 * self.axis.2
                - 1.0f32)
                .abs()
                < 1e-5,
            "RevoluteJoint axis should be a unit vector"
        );
        let t = new_translation(&self.translation);
        let r = self.rotation_matrix();
        &t * &r
    }

    fn dof_count(&self) -> usize {
        1
    }

    fn pack(&self, out: &mut [f32]) {
        debug_assert!(
            out.len() >= self.dof_count(),
            "pack slice length >= dof_count"
        );
        if !out.is_empty() {
            out[0] = self.angle;
        }
    }

    fn unpack(&mut self, data: &[f32]) {
        debug_assert!(
            data.len() >= self.dof_count(),
            "unpack slice length >= dof_count"
        );
        if !data.is_empty() {
            self.angle = data[0];
            if let (Some(lo), Some(hi)) = (self.angle_min, self.angle_max) {
                self.angle = self.angle.clamp(lo, hi);
            }
        }
    }
}
