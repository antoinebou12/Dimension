//! Spherical joint: 3 DOF rotation (quaternion or Euler).

use super::Joint;
use mathlib::cg::new_translation;
use mathlib::{Matrix4f, Quat4f, Vector3f};

/// Spherical joint: 3 DOF rotation (full 3D orientation).
#[derive(Clone, Debug)]
pub struct SphericalJoint {
    /// Local translation offset.
    pub translation: Vector3f,
    /// Orientation as quaternion (w, x, y, z).
    pub quat: Quat4f,
}

impl Default for SphericalJoint {
    fn default() -> Self {
        let mut t = Vector3f::with_capacity(3);
        t.set_zero();
        Self {
            translation: t,
            quat: Quat4f::identity(),
        }
    }
}

impl SphericalJoint {
    /// Creates a spherical joint.
    #[must_use]
    pub fn new(translation: Vector3f, quat: Quat4f) -> Self {
        Self { translation, quat }
    }

    /// Creates from Euler angles (roll, pitch, yaw) in radians.
    #[must_use]
    pub fn from_euler(translation: Vector3f, roll: f32, pitch: f32, yaw: f32) -> Self {
        Self {
            translation,
            quat: Quat4f::from_euler_angles(roll, pitch, yaw),
        }
    }
}

impl Joint for SphericalJoint {
    fn local_transform(&self) -> Matrix4f {
        let t = new_translation(&self.translation);
        let r = self.quat.to_rotation_matrix4();
        &t * &r
    }

    fn dof_count(&self) -> usize {
        3
    }

    fn pack(&self, out: &mut [f32]) {
        debug_assert!(
            out.len() >= self.dof_count(),
            "pack slice length >= dof_count"
        );
        if out.len() >= 3 {
            let sa = self.quat.to_scaled_axis();
            out[0] = sa.get(0);
            out[1] = sa.get(1);
            out[2] = sa.get(2);
        }
    }

    fn unpack(&mut self, data: &[f32]) {
        debug_assert!(
            data.len() >= self.dof_count(),
            "unpack slice length >= dof_count"
        );
        if data.len() >= 3 {
            let x = data[0];
            let y = data[1];
            let z = data[2];
            let angle_sq = x * x + y * y + z * z;
            const EPS_SQ: f32 = 1e-14;
            if angle_sq < EPS_SQ {
                self.quat = Quat4f::identity();
                return;
            }
            let angle = angle_sq.sqrt();
            let mut axis = Vector3f::with_capacity(3);
            axis.set(0, x / angle);
            axis.set(1, y / angle);
            axis.set(2, z / angle);
            self.quat = Quat4f::from_axis_angle(&axis, angle);
        }
    }
}
