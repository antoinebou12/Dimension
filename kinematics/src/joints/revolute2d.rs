//! Revolute joint in 2D: 1 DOF rotation about Z axis.

use super::Joint;
use mathlib::cg::{from_scaled_axis, new_translation};
use mathlib::{Matrix4f, Vector3f};

/// Revolute joint in the XY plane: rotation about Z axis (0, 0, 1), 1 DOF.
#[derive(Clone, Debug)]
pub struct Revolute2dJoint {
    /// Local translation (x, y, 0) before rotation.
    pub translation: Vector3f,
    /// Angle in radians (rotation about Z).
    pub angle: f32,
    /// Optional minimum angle (radians). When set with [`Self::angle_max`], angle is clamped in [`Self::local_transform`] and [`Self::unpack`].
    pub angle_min: Option<f32>,
    /// Optional maximum angle (radians).
    pub angle_max: Option<f32>,
}

impl Default for Revolute2dJoint {
    fn default() -> Self {
        let mut t = Vector3f::with_capacity(3);
        t.set_zero();
        Self {
            translation: t,
            angle: 0.0,
            angle_min: None,
            angle_max: None,
        }
    }
}

impl Revolute2dJoint {
    /// Creates a 2D revolute joint at (x, y) with given angle.
    #[must_use]
    pub fn new(translation: Vector3f, angle: f32) -> Self {
        Self {
            translation,
            angle,
            angle_min: None,
            angle_max: None,
        }
    }

    /// Creates a 2D revolute joint at (x, y) with z = 0.
    #[must_use]
    pub fn at(x: f32, y: f32, angle: f32) -> Self {
        let mut t = Vector3f::with_capacity(3);
        t.set(0, x);
        t.set(1, y);
        t.set(2, 0.0);
        Self {
            translation: t,
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

    fn clamped_angle(&self) -> f32 {
        match (self.angle_min, self.angle_max) {
            (Some(lo), Some(hi)) => self.angle.clamp(lo, hi),
            _ => self.angle,
        }
    }

    fn rotation_matrix_z(&self) -> Matrix4f {
        let a = self.clamped_angle();
        let mut axis_angle = Vector3f::with_capacity(3);
        axis_angle.set(0, 0.0);
        axis_angle.set(1, 0.0);
        axis_angle.set(2, a);
        from_scaled_axis(&axis_angle)
    }
}

impl Joint for Revolute2dJoint {
    fn local_transform(&self) -> Matrix4f {
        let t = new_translation(&self.translation);
        let r = self.rotation_matrix_z();
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

#[cfg(test)]
mod tests {
    use super::{Joint, Revolute2dJoint};
    use std::f32::consts::FRAC_PI_2;

    #[test]
    fn revolute2d_dof_pack_unpack() {
        let mut j = Revolute2dJoint::at(0.0, 0.0, 0.5);
        assert_eq!(j.dof_count(), 1);
        let mut out = [0.0];
        j.pack(&mut out);
        assert!((out[0] - 0.5).abs() < 1e-6);
        j.unpack(&[FRAC_PI_2]);
        assert!((j.angle - FRAC_PI_2).abs() < 1e-6);
    }

    #[test]
    fn revolute2d_transform_at_origin() {
        let j = Revolute2dJoint::at(0.0, 0.0, 0.0);
        let m = j.local_transform();
        assert!((m.get(0, 3)).abs() < 1e-5);
        assert!((m.get(1, 3)).abs() < 1e-5);
    }
}
