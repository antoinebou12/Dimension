//! Soft body: tetrahedral mesh with Neo-Hookean material parameters.
//!
//! A [`SoftBody`] owns a [`TetMesh`] and material parameters (Lamé μ, λ).
//! Its vertices are stored as particles in [`PhysicsState::particles`],
//! starting at `particle_offset`.

use crate::tet::TetMesh;

/// Soft body — a deformable volumetric object.
///
/// Vertices are shared with the particle array so the existing solver
/// infrastructure handles prediction and velocity update. The Neo-Hookean
/// constraints in [`crate::neohookean`] operate on the tetrahedra.
///
/// # Material parameters
///
/// Lamé parameters μ (shear modulus) and λ (bulk modulus) control stiffness.
/// Convert from Young's modulus E and Poisson's ratio ν:
///
/// ```text
/// μ = E / (2 (1 + ν))
/// λ = E ν / ((1 + ν)(1 − 2ν))
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SoftBody {
    /// Index into `PhysicsState::particles` for the first vertex.
    pub particle_offset: usize,
    /// Number of vertices owned by this soft body.
    pub num_vertices: usize,
    /// Tetrahedral mesh topology and rest-pose data.
    pub mesh: TetMesh,
    /// First Lamé parameter μ (shear modulus).
    pub mu: f32,
    /// Second Lamé parameter λ (bulk modulus).
    pub lambda: f32,
    /// Phase identifier for collision grouping.
    pub phase: u32,
}

impl SoftBody {
    /// Create a soft body from material and mesh data.
    ///
    /// # Arguments
    ///
    /// * `particle_offset` — index of first vertex in the global particle array.
    /// * `num_vertices` — number of vertices.
    /// * `mesh` — tetrahedral mesh with precomputed Dm⁻¹ and rest volumes.
    /// * `mu` — first Lamé parameter (shear modulus).
    /// * `lambda` — second Lamé parameter (bulk modulus).
    #[must_use]
    pub fn new(
        particle_offset: usize,
        num_vertices: usize,
        mesh: TetMesh,
        mu: f32,
        lambda: f32,
    ) -> Self {
        Self {
            particle_offset,
            num_vertices,
            mesh,
            mu,
            lambda,
            phase: 0,
        }
    }

    /// Create a soft body from Young's modulus E and Poisson's ratio ν.
    #[must_use]
    pub fn from_youngs(
        particle_offset: usize,
        num_vertices: usize,
        mesh: TetMesh,
        young: f32,
        poisson: f32,
    ) -> Self {
        let (mu, lambda) = lame_from_youngs(young, poisson);
        Self::new(particle_offset, num_vertices, mesh, mu, lambda)
    }

    /// Sets the phase identifier.
    #[must_use]
    pub fn with_phase(mut self, phase: u32) -> Self {
        self.phase = phase;
        self
    }

    /// Compute XPBD compliance for the deviatoric constraint of tet `i`.
    ///
    /// `α̃_dev = 1 / (μ · V₀ · Δτ²)`
    #[must_use]
    pub fn compliance_deviatoric(&self, tet_idx: usize, dt_sq: f32) -> f32 {
        let vol = self.mesh.rest_volumes[tet_idx];
        let denom = self.mu * vol * dt_sq;
        if denom.abs() < 1e-20 {
            return 0.0;
        }
        1.0 / denom
    }

    /// Compute XPBD compliance for the hydrostatic constraint of tet `i`.
    ///
    /// `α̃_hydro = 1 / (λ · V₀ · Δτ²)`
    #[must_use]
    pub fn compliance_hydrostatic(&self, tet_idx: usize, dt_sq: f32) -> f32 {
        let vol = self.mesh.rest_volumes[tet_idx];
        let denom = self.lambda * vol * dt_sq;
        if denom.abs() < 1e-20 {
            return 0.0;
        }
        1.0 / denom
    }

    /// Global particle index for local vertex `v` of this soft body.
    #[inline]
    #[must_use]
    pub fn global_index(&self, local: usize) -> usize {
        self.particle_offset + local
    }
}

/// Convert Young's modulus E and Poisson's ratio ν to Lamé parameters (μ, λ).
#[must_use]
pub fn lame_from_youngs(young: f32, poisson: f32) -> (f32, f32) {
    let mu = young / (2.0 * (1.0 + poisson));
    let lambda = young * poisson / ((1.0 + poisson) * (1.0 - 2.0 * poisson));
    (mu, lambda)
}

/// Preset material parameters.
pub mod presets {
    /// Soft rubber (E ≈ 1e5, ν ≈ 0.45).
    pub const RUBBER_SOFT: (f32, f32) = (3.4e4, 6.9e5);
    /// Stiff rubber (E ≈ 1e7, ν ≈ 0.48).
    pub const RUBBER_STIFF: (f32, f32) = (3.4e6, 8.0e7);
    /// Flesh (E ≈ 5e4, ν ≈ 0.40).
    pub const FLESH: (f32, f32) = (1.8e4, 1.4e5);
    /// Jelly (E ≈ 1e3, ν ≈ 0.49).
    pub const JELLY: (f32, f32) = (3.4e2, 1.6e4);
}
