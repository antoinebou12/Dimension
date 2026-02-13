//! Joint types for kinematic chains.
//!
//! Each joint type defines its local transform and degrees of freedom (DOF).
//! 2D joints (Fixed2d, Revolute2d, Prismatic2d) operate in the XY plane (z = 0).

mod fixed;
mod fixed2d;
mod prismatic;
mod prismatic2d;
mod revolute;
mod revolute2d;
mod spherical;

pub use fixed::FixedJoint;
pub use fixed2d::Fixed2dJoint;
pub use prismatic::PrismaticJoint;
pub use prismatic2d::Prismatic2dJoint;
pub use revolute::RevoluteJoint;
pub use revolute2d::Revolute2dJoint;
pub use spherical::SphericalJoint;

use mathlib::Matrix4f;

/// Trait for kinematic joints: local transform and DOF.
pub trait Joint {
    /// Local transform matrix (4×4) from joint state.
    fn local_transform(&self) -> Matrix4f;

    /// Number of degrees of freedom.
    fn dof_count(&self) -> usize;

    /// Pack joint state into a slice (caller ensures length >= dof_count).
    fn pack(&self, out: &mut [f32]);

    /// Unpack joint state from a slice.
    fn unpack(&mut self, data: &[f32]);
}
