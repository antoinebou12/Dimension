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
        let (r, p, y) = self.quat.to_euler_angles();
        if out.len() >= 3 {
            out[0] = r;
            out[1] = p;
            out[2] = y;
        }
    }

    fn unpack(&mut self, data: &[f32]) {
        debug_assert!(
            data.len() >= self.dof_count(),
            "unpack slice length >= dof_count"
        );
        if data.len() >= 3 {
            self.quat = Quat4f::from_euler_angles(data[0], data[1], data[2]);
        }
    }
}
