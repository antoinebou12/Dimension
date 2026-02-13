//! Integrator trait and implementations (PBD, XPBD, explicit Euler).

use crate::body_constraint::{PositionalConstraint, RigidContactConstraint};
use crate::constraint::Constraint;
use crate::hooks::SubstepHooks;
use crate::state::PhysicsState;

/// Integrator for advancing physics state by a time step.
pub trait Integrator: Send + Sync {
    /// Advances the state by `dt` seconds.
    fn step(&mut self, state: &mut PhysicsState, dt: f32);
}

/// PBD integrator with particle constraints only.
#[derive(Default)]
pub struct PbdIntegrator {
    particle_constraints: Vec<Box<dyn Constraint>>,
    hooks: SubstepHooks,
}

impl PbdIntegrator {
    /// Creates a new PBD integrator.
    #[must_use]
    pub fn new() -> Self {
        Self {
            particle_constraints: Vec::new(),
            hooks: SubstepHooks::default(),
        }
    }

    /// Add a particle constraint.
    pub fn add_constraint(&mut self, c: Box<dyn Constraint>) {
        self.particle_constraints.push(c);
    }

    /// Access hooks.
    pub fn hooks_mut(&mut self) -> &mut SubstepHooks {
        &mut self.hooks
    }
}

impl Integrator for PbdIntegrator {
    fn step(&mut self, state: &mut PhysicsState, dt: f32) {
        crate::solver::step_xpbd(
            state,
            &self.particle_constraints,
            &mut [],
            &mut [],
            dt,
            &mut self.hooks,
        );
    }
}

/// Full XPBD integrator: particles, rigid bodies, and soft bodies.
#[derive(Default)]
pub struct XpbdIntegrator {
    /// Particle-level constraints.
    pub particle_constraints: Vec<Box<dyn Constraint>>,
    /// Rigid body positional constraints (ball joints, etc.).
    pub positional_constraints: Vec<PositionalConstraint>,
    /// Rigid body contact constraints.
    pub rigid_contacts: Vec<RigidContactConstraint>,
    /// Substep hooks.
    pub hooks: SubstepHooks,
}

impl XpbdIntegrator {
    /// Creates a new XPBD integrator.
    #[must_use]
    pub fn new() -> Self {
        Self {
            particle_constraints: Vec::new(),
            positional_constraints: Vec::new(),
            rigid_contacts: Vec::new(),
            hooks: SubstepHooks::default(),
        }
    }

    /// Add a particle constraint.
    pub fn add_particle_constraint(&mut self, c: Box<dyn Constraint>) {
        self.particle_constraints.push(c);
    }

    /// Add a rigid body positional constraint.
    pub fn add_positional_constraint(&mut self, c: PositionalConstraint) {
        self.positional_constraints.push(c);
    }

    /// Clear transient contact constraints.
    pub fn clear_contacts(&mut self) {
        self.rigid_contacts.clear();
    }
}

impl Integrator for XpbdIntegrator {
    fn step(&mut self, state: &mut PhysicsState, dt: f32) {
        crate::solver::step_xpbd(
            state,
            &self.particle_constraints,
            &mut self.positional_constraints,
            &mut self.rigid_contacts,
            dt,
            &mut self.hooks,
        );
    }
}

/// Explicit (semi-implicit) Euler integrator.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ExplicitEulerIntegrator;

impl ExplicitEulerIntegrator {
    /// Creates a new explicit Euler integrator.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Integrator for ExplicitEulerIntegrator {
    fn step(&mut self, state: &mut PhysicsState, dt: f32) {
        crate::solver::step_explicit_euler(state, dt);
    }
}
