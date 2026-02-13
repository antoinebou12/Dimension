//! Fixed joint in 2D: translation only (x, y, 0), 0 DOF.

use super::Joint;
use mathlib::cg::new_translation;
use mathlib::{Matrix4f, Vector3f};

/// Fixed joint in the XY plane: constant translation (x, y, 0), 0 DOF.
#[derive(Clone, Debug)]
pub struct Fixed2dJoint {
    /// Local translation (x, y); z is implicitly 0.
    pub translation: Vector3f,
}

impl Default for Fixed2dJoint {
    fn default() -> Self {
        let mut t = Vector3f::with_capacity(3);
        t.set_zero();
        Self { translation: t }
    }
}

impl Fixed2dJoint {
    /// Creates a 2D fixed joint at (x, y).
    #[must_use]
    pub fn new(translation: Vector3f) -> Self {
        Self { translation }
    }

    /// Creates a 2D fixed joint at (x, y) with z = 0.
    #[must_use]
    pub fn at(x: f32, y: f32) -> Self {
        let mut t = Vector3f::with_capacity(3);
        t.set(0, x);
        t.set(1, y);
        t.set(2, 0.0);
        Self { translation: t }
    }
}

impl Joint for Fixed2dJoint {
    fn local_transform(&self) -> Matrix4f {
        new_translation(&self.translation)
    }

    fn dof_count(&self) -> usize {
        0
    }

    fn pack(&self, _out: &mut [f32]) {}

    fn unpack(&mut self, _data: &[f32]) {}
}

#[cfg(test)]
mod tests {
    use super::{Fixed2dJoint, Joint};

    #[test]
    fn fixed2d_dof_and_transform() {
        let j = Fixed2dJoint::at(1.0, 2.0);
        assert_eq!(j.dof_count(), 0);
        let m = j.local_transform();
        assert!((m.get(0, 3) - 1.0).abs() < 1e-5);
        assert!((m.get(1, 3) - 2.0).abs() < 1e-5);
        assert!((m.get(2, 3) - 0.0).abs() < 1e-5);
    }
}
