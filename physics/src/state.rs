//! Physics state: particles, rigid bodies, soft bodies, and configuration.

use crate::body::RigidBody;
use crate::particle::Particle;
use crate::softbody::SoftBody;

/// Solver configuration.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PhysicsConfig {
    /// Time step per frame (seconds).
    pub dt: f32,
    /// Number of substeps per frame.
    pub substeps: u32,
    /// Solver iterations per substep.
    pub solver_iterations: u32,
    /// Pre-stabilization iterations for contacts.
    pub stabilization_iterations: u32,
    /// SOR factor omega in \[1, 2\].
    pub sor_omega: f32,
    /// Sleep threshold: freeze if displacement below this.
    pub sleep_threshold: f32,
    /// Gravity vector applied to all dynamic bodies each substep.
    pub gravity: [f32; 3],
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            dt: 1.0 / 60.0,
            substeps: 2,
            solver_iterations: 4,
            stabilization_iterations: 1,
            sor_omega: 1.2,
            sleep_threshold: 1e-6,
            gravity: [0.0, -9.81, 0.0],
        }
    }
}

/// Aggregated physics state: all bodies and solver config.
///
/// Constraints are held separately in the integrator so that the state
/// remains serializable and easy to snapshot.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PhysicsState {
    /// Point-mass particles (also serves as soft body vertices).
    pub particles: Vec<Particle>,
    /// Rigid bodies with orientation and inertia.
    pub rigid_bodies: Vec<RigidBody>,
    /// Soft bodies (tetrahedral mesh referencing particles).
    pub soft_bodies: Vec<SoftBody>,
    /// Configuration (dt, substeps, gravity, etc.).
    pub config: PhysicsConfig,
}

impl PhysicsState {
    /// Creates an empty state with default config.
    #[must_use]
    pub fn new() -> Self {
        Self {
            particles: Vec::new(),
            rigid_bodies: Vec::new(),
            soft_bodies: Vec::new(),
            config: PhysicsConfig::default(),
        }
    }

    /// Creates state with the given particles and default config.
    #[must_use]
    pub fn with_particles(particles: Vec<Particle>) -> Self {
        Self {
            particles,
            rigid_bodies: Vec::new(),
            soft_bodies: Vec::new(),
            config: PhysicsConfig::default(),
        }
    }

    /// Returns the number of particles.
    #[must_use]
    pub fn num_particles(&self) -> usize {
        self.particles.len()
    }

    /// Returns the number of rigid bodies.
    #[must_use]
    pub fn num_rigid_bodies(&self) -> usize {
        self.rigid_bodies.len()
    }

    /// Returns the number of soft bodies.
    #[must_use]
    pub fn num_soft_bodies(&self) -> usize {
        self.soft_bodies.len()
    }

    /// Add a rigid body and return its index.
    pub fn add_rigid_body(&mut self, body: RigidBody) -> usize {
        let idx = self.rigid_bodies.len();
        self.rigid_bodies.push(body);
        idx
    }

    /// Add a soft body whose vertices are already in `self.particles`.
    pub fn add_soft_body(&mut self, body: SoftBody) -> usize {
        let idx = self.soft_bodies.len();
        self.soft_bodies.push(body);
        idx
    }
}

impl Default for PhysicsState {
    fn default() -> Self {
        Self::new()
    }
}
