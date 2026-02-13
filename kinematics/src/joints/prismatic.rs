//! Prismatic joint: 1 DOF translation along an axis.

use super::Joint;
use mathlib::cg::{from_euler_angles, new_translation};
use mathlib::{Matrix4f, Vector3f};

/// Prismatic joint: 1 DOF translation along axis (default X).
#[derive(Clone, Debug)]
pub struct PrismaticJoint {
    /// Base translation offset.
    pub translation: Vector3f,
    /// Direction of slide (x, y, z), unit vector.
    pub axis: (f32, f32, f32),
    /// Local rotation (Euler) before slide.
    pub rotation: (f32, f32, f32),
    /// Slide amount (scalar along axis).
    pub offset: f32,
}

impl Default for PrismaticJoint {
    fn default() -> Self {
        let mut t = Vector3f::with_capacity(3);
        t.set_zero();
        Self {
            translation: t,
            axis: (1.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            offset: 0.0,
        }
    }
}

impl PrismaticJoint {
    /// Creates a prismatic joint.
    #[must_use]
    pub fn new(
        translation: Vector3f,
        axis: (f32, f32, f32),
        rotation: (f32, f32, f32),
        offset: f32,
    ) -> Self {
        Self {
            translation,
            axis,
            rotation,
            offset,
        }
    }
}

impl Joint for PrismaticJoint {
    fn local_transform(&self) -> Matrix4f {
        debug_assert!(
            (self.axis.0 * self.axis.0 + self.axis.1 * self.axis.1 + self.axis.2 * self.axis.2
                - 1.0f32)
                .abs()
                < 1e-5,
            "PrismaticJoint axis should be a unit vector"
        );
        let r = from_euler_angles(self.rotation.0, self.rotation.1, self.rotation.2);
        let mut slide = Vector3f::with_capacity(3);
        slide.set(0, self.translation.get(0) + self.axis.0 * self.offset);
        slide.set(1, self.translation.get(1) + self.axis.1 * self.offset);
        slide.set(2, self.translation.get(2) + self.axis.2 * self.offset);
        let t = new_translation(&slide);
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
            out[0] = self.offset;
        }
    }

    fn unpack(&mut self, data: &[f32]) {
        debug_assert!(
            data.len() >= self.dof_count(),
            "unpack slice length >= dof_count"
        );
        if !data.is_empty() {
            self.offset = data[0];
        }
    }
}
