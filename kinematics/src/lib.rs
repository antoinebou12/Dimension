//! kinematics — Forward and inverse kinematics for articulated structures.
//!
//! Uses mathlib Tree for hierarchy, multiple joint types (fixed, revolute, prismatic, spherical),
//! forward kinematics, pack/unpack, reroot, and IK solvers (Jacobian, FABRIK).

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

pub mod armature;
pub mod ik;
pub mod joints;

#[cfg(feature = "wasm")]
pub mod wasm;

pub use armature::{Armature, JointData, JointVariant};
#[cfg(not(target_arch = "wasm32"))]
pub use ik::HalleyIk;
pub use ik::{FabrikIk, JacobianIk};
pub use joints::{
    Fixed2dJoint, FixedJoint, Joint, Prismatic2dJoint, PrismaticJoint, Revolute2dJoint,
    RevoluteJoint, SphericalJoint,
};
