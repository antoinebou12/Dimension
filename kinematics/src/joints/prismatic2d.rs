//! Prismatic joint in 2D: 1 DOF translation along an axis in the XY plane.

use super::Joint;
use mathlib::cg::new_translation;
use mathlib::{Matrix4f, Vector3f};

/// Prismatic joint in the XY plane: slide along axis (ax, ay, 0), 1 DOF.
#[derive(Clone, Debug)]
pub struct Prismatic2dJoint {
    /// Base translation (x, y, 0).
    pub translation: Vector3f,
    /// Direction of slide in XY (unit vector); z is 0.
    pub axis: (f32, f32),
    /// Slide amount along axis.
    pub offset: f32,
}

impl Default for Prismatic2dJoint {
    fn default() -> Self {
        let mut t = Vector3f::with_capacity(3);
        t.set_zero();
        Self {
            translation: t,
            axis: (1.0, 0.0),
            offset: 0.0,
        }
    }
}

impl Prismatic2dJoint {
    /// Creates a 2D prismatic joint.
    #[must_use]
    pub fn new(translation: Vector3f, axis: (f32, f32), offset: f32) -> Self {
        Self {
            translation,
            axis,
            offset,
        }
    }

    /// Creates a 2D prismatic joint at (x, y) with axis (ax, ay).
    #[must_use]
    pub fn at(x: f32, y: f32, axis: (f32, f32), offset: f32) -> Self {
        let mut t = Vector3f::with_capacity(3);
        t.set(0, x);
        t.set(1, y);
        t.set(2, 0.0);
        Self {
            translation: t,
            axis,
            offset,
        }
    }
}

impl Joint for Prismatic2dJoint {
    fn local_transform(&self) -> Matrix4f {
        let axis_len_sq = self.axis.0 * self.axis.0 + self.axis.1 * self.axis.1;
        debug_assert!(
            (axis_len_sq - 1.0f32).abs() < 1e-5,
            "Prismatic2dJoint axis (ax, ay) should be a unit vector in XY"
        );
        let mut slide = Vector3f::with_capacity(3);
        slide.set(0, self.translation.get(0) + self.axis.0 * self.offset);
        slide.set(1, self.translation.get(1) + self.axis.1 * self.offset);
        slide.set(2, 0.0);
        new_translation(&slide)
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

#[cfg(test)]
mod tests {
    use super::{Joint, Prismatic2dJoint};

    #[test]
    fn prismatic2d_dof_pack_unpack() {
        let mut j = Prismatic2dJoint::at(0.0, 0.0, (1.0, 0.0), 2.0);
        assert_eq!(j.dof_count(), 1);
        let mut out = [0.0];
        j.pack(&mut out);
        assert!((out[0] - 2.0).abs() < 1e-6);
        j.unpack(&[3.0]);
        assert!((j.offset - 3.0).abs() < 1e-6);
    }

    #[test]
    fn prismatic2d_transform() {
        let j = Prismatic2dJoint::at(1.0, 0.0, (1.0, 0.0), 2.0);
        let m = j.local_transform();
        assert!((m.get(0, 3) - 3.0).abs() < 1e-5);
        assert!((m.get(1, 3) - 0.0).abs() < 1e-5);
    }
}
