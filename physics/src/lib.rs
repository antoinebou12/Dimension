//! Position-based dynamics (PBD/XPBD) physics for real-time applications.
//!
//! Supports three body types:
//!
//! - **Particles** — point masses connected by constraints (distance, pin)
//! - **Rigid bodies** — orientation-based with quaternion, inertia, collision shapes
//! - **Soft bodies** — tetrahedral meshes with Neo-Hookean elasticity constraints
//!
//! All body types share one unified XPBD solver loop with compliance-based
//! stiffness, constraint averaging, and sleeping.
//!
//! # Examples
//!
//! ```
//! use physics::{Particle, PhysicsState, PbdIntegrator, Integrator};
//!
//! let mut state = PhysicsState::with_particles(vec![
//!     Particle::new([0.0, 0.0, 0.0], 1.0),
//!     Particle::new([1.0, 0.0, 0.0], 1.0),
//! ]);
//! state.config.gravity = [0.0, 0.0, 0.0]; // disable gravity for this test
//! let mut integrator = PbdIntegrator::new();
//! integrator.step(&mut state, 1.0 / 60.0);
//! ```

pub mod body;
pub mod body_constraint;
pub mod constraint;
pub mod contact;
pub mod hooks;
pub mod integration;
pub mod neohookean;
pub mod particle;
pub mod softbody;
pub mod solver;
pub mod state;
pub mod tet;

#[cfg(feature = "serde")]
pub mod serialization;

// Re-exports
pub use body::{CollisionShape, RigidBody};
pub use body_constraint::{PositionalConstraint, RigidContactConstraint};
pub use constraint::{Constraint, DistanceConstraint, PinConstraint, PlaneConstraint};
pub use contact::ContactConstraint;
pub use hooks::SubstepHooks;
pub use integration::{ExplicitEulerIntegrator, Integrator, PbdIntegrator, XpbdIntegrator};
pub use neohookean::TetSolveResult;
pub use particle::Particle;
pub use softbody::SoftBody;
pub use state::{PhysicsConfig, PhysicsState};
pub use tet::TetMesh;
