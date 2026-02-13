//! Particle state: position, velocity, inverse mass, and phase for grouping.

/// Particle state used as the fundamental building block for all object types.
///
/// Position and velocity are 3D; for 2D simulations, the z component can be ignored or fixed.
/// The `phase` identifier groups particles (e.g. to disable collisions between groups).
/// Use `radius` > 0 for sphere collision (contact constraints).
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Particle {
    /// Position (x, y, z).
    pub x: [f32; 3],
    /// Velocity (vx, vy, vz).
    pub v: [f32; 3],
    /// Inverse mass (1/m). Use 0 for kinematic (fixed) particles.
    pub inv_mass: f32,
    /// Phase identifier for grouping; e.g. disable collisions between different phases.
    pub phase: u32,
    /// Collision radius (sphere). Use 0 for no collision.
    pub radius: f32,
}

impl Particle {
    /// Creates a particle at the given position with zero velocity.
    ///
    /// # Examples
    ///
    /// ```
    /// # use physics::Particle;
    /// let p = Particle::new([0.0, 1.0, 0.0], 1.0);
    /// assert_eq!(p.x, [0.0, 1.0, 0.0]);
    /// assert_eq!(p.inv_mass, 1.0);
    /// assert_eq!(p.phase, 0);
    /// ```
    #[must_use]
    pub fn new(x: [f32; 3], inv_mass: f32) -> Self {
        Self {
            x,
            v: [0.0; 3],
            inv_mass,
            phase: 0,
            radius: 0.0,
        }
    }

    /// Creates a kinematic (fixed) particle at the given position.
    #[must_use]
    pub fn kinematic(x: [f32; 3]) -> Self {
        Self {
            x,
            v: [0.0; 3],
            inv_mass: 0.0,
            phase: 0,
            radius: 0.0,
        }
    }

    /// Sets the phase identifier.
    #[must_use]
    pub fn with_phase(mut self, phase: u32) -> Self {
        self.phase = phase;
        self
    }

    /// Sets the collision radius (sphere). Use 0 for no collision.
    #[must_use]
    pub fn with_radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    /// Returns true if this particle is kinematic (infinite mass).
    #[must_use]
    pub fn is_kinematic(&self) -> bool {
        self.inv_mass <= 0.0
    }
}

impl Default for Particle {
    fn default() -> Self {
        Self {
            x: [0.0; 3],
            v: [0.0; 3],
            inv_mass: 1.0,
            phase: 0,
            radius: 0.0,
        }
    }
}
