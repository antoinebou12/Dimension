//! Fixed joint: no degrees of freedom, constant transform.

use super::Joint;
use mathlib::cg::{from_euler_angles, new_translation};
use mathlib::{Matrix4f, Vector3f};

/// Fixed joint: constant local transform, 0 DOF.
#[derive(Clone, Debug)]
pub struct FixedJoint {
    /// Local translation (x, y, z).
    pub translation: Vector3f,
    /// Local rotation as Euler (roll, pitch, yaw) in radians.
    pub rotation: (f32, f32, f32),
}

impl Default for FixedJoint {
    fn default() -> Self {
        let mut t = Vector3f::with_capacity(3);
        t.set_zero();
        Self {
            translation: t,
            rotation: (0.0, 0.0, 0.0),
        }
    }
}

impl FixedJoint {
    /// Creates a fixed joint with given translation and Euler rotation.
    #[must_use]
    pub fn new(translation: Vector3f, rotation: (f32, f32, f32)) -> Self {
        Self {
            translation,
            rotation,
        }
    }
}

impl Joint for FixedJoint {
    fn local_transform(&self) -> Matrix4f {
        let r = from_euler_angles(self.rotation.0, self.rotation.1, self.rotation.2);
        let t = new_translation(&self.translation);
        &t * &r
    }

    fn dof_count(&self) -> usize {
        0
    }

    fn pack(&self, _out: &mut [f32]) {}

    fn unpack(&mut self, _data: &[f32]) {}
}
