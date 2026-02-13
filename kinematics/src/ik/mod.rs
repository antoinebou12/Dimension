//! Inverse kinematics solvers.

mod chain;
mod fabrik;
#[cfg(not(target_arch = "wasm32"))]
mod halley;
mod jacobian;

pub use fabrik::FabrikIk;
#[cfg(not(target_arch = "wasm32"))]
pub use halley::HalleyIk;
pub use jacobian::JacobianIk;
