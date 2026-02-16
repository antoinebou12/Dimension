//! kinematics — Forward and inverse kinematics for articulated structures.
//!
//! Uses mathlib Tree for hierarchy, multiple joint types (fixed, revolute, prismatic, spherical),
//! forward kinematics, pack/unpack, reroot, and IK solvers ([FabrikIk], [JacobianIk], [HalleyIk]).
//! See the crate README for IK solver comparison and references.

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

pub mod armature;
pub mod ik;
pub mod joints;

#[cfg(feature = "wasm")]
pub mod wasm;

pub use armature::{Armature, JointData, JointVariant};
pub use ik::HalleyIk;
pub use ik::{
    CcdIk, FabrikIk, FabrikSqpIk, HessianIk, JacobianIk, hessian_snapshot, solve_batch_halley,
    solve_batch_hessian, solve_batch_jacobian,
};
pub use joints::{
    Fixed2dJoint, FixedJoint, Joint, Prismatic2dJoint, PrismaticJoint, Revolute2dJoint,
    RevoluteJoint, SphericalJoint,
};
