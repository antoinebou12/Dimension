//! Inverse kinematics solvers.
//!
//! Solvers: [FabrikIk] (position-only, all chain types, WASM), [JacobianIk] (position-only, SVD),
//! [HessianIk] (position-only, exact Hessian Newton, WASM), [HalleyIk] (6D pose, serial chains). See the crate README and AGENTS.md for
//! comparison and when to use each.

mod batch;
mod ccd;
mod chain;
mod fabrik;
mod fabrik_sqp;
mod halley;
mod hessian;
mod jacobian;
mod util;

pub use batch::{solve_batch_halley, solve_batch_hessian, solve_batch_jacobian};
pub use ccd::CcdIk;
pub use fabrik::{FabrikIk, JointConeConstraint};
pub use fabrik_sqp::FabrikSqpIk;
pub use halley::HalleyIk;
pub use hessian::{HessianIk, hessian_snapshot};
pub use jacobian::JacobianIk;
