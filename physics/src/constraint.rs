//! Constraint trait and built-in constraint types (distance, pin, plane).

use crate::particle::Particle;

/// Constraint that can be solved in the PBD/XPBD loop.
///
/// Constraints reference particles by index and compute position deltas
/// to satisfy the constraint (e.g. distance, pin to target).
pub trait Constraint: Send + Sync {
    /// Number of particles this constraint affects.
    fn num_particles(&self) -> usize;

    /// Particle index for the k-th participant (k in 0..num_particles()).
    fn particle_index(&self, k: usize) -> usize;

    /// Evaluate constraint and accumulate position deltas into `dx`.
    /// `particles` and `predicted` are the same length; use `predicted` for
    /// the current candidate positions. `dx[i]` is the accumulated delta for
    /// particle index `i`; add this constraint's contribution.
    /// `n_affecting[i]` is the number of constraints affecting particle `i` (for averaging).
    fn solve(
        &self,
        particles: &[Particle],
        predicted: &[[f32; 3]],
        dx: &mut [[f32; 3]],
        n_affecting: &mut [u32],
    );
}

/// Distance constraint: keeps two particles at a target distance.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DistanceConstraint {
    /// Index of first particle.
    pub i: usize,
    /// Index of second particle.
    pub j: usize,
    /// Rest (target) distance.
    pub rest_length: f32,
}

impl DistanceConstraint {
    /// Creates a distance constraint between particles `i` and `j` with the given rest length.
    #[must_use]
    pub fn new(i: usize, j: usize, rest_length: f32) -> Self {
        Self { i, j, rest_length }
    }
}

impl Constraint for DistanceConstraint {
    fn num_particles(&self) -> usize {
        2
    }

    fn particle_index(&self, k: usize) -> usize {
        match k {
            0 => self.i,
            1 => self.j,
            _ => self.i,
        }
    }

    fn solve(
        &self,
        _particles: &[Particle],
        predicted: &[[f32; 3]],
        dx: &mut [[f32; 3]],
        n_affecting: &mut [u32],
    ) {
        let pi = predicted[self.i];
        let pj = predicted[self.j];
        let dx_ij = [pi[0] - pj[0], pi[1] - pj[1], pi[2] - pj[2]];
        let dist_sq = dx_ij[0] * dx_ij[0] + dx_ij[1] * dx_ij[1] + dx_ij[2] * dx_ij[2];
        let dist = dist_sq.sqrt().max(1e-8);
        let diff = dist - self.rest_length;
        let n_inv = 1.0 / dist;
        let nx = dx_ij[0] * n_inv;
        let ny = dx_ij[1] * n_inv;
        let nz = dx_ij[2] * n_inv;

        let wi = _particles[self.i].inv_mass;
        let wj = _particles[self.j].inv_mass;
        let w_sum = wi + wj;
        if w_sum <= 0.0 {
            return;
        }
        let scale = diff / w_sum;
        let dxi = scale * wi;
        let dxj = scale * wj;

        dx[self.i][0] -= dxi * nx;
        dx[self.i][1] -= dxi * ny;
        dx[self.i][2] -= dxi * nz;
        dx[self.j][0] += dxj * nx;
        dx[self.j][1] += dxj * ny;
        dx[self.j][2] += dxj * nz;
        n_affecting[self.i] = n_affecting[self.i].saturating_add(1);
        n_affecting[self.j] = n_affecting[self.j].saturating_add(1);
    }
}

/// Pin constraint: keeps a particle at a target position.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PinConstraint {
    /// Index of the particle to pin.
    pub i: usize,
    /// Target position (x, y, z).
    pub target: [f32; 3],
}

impl PinConstraint {
    /// Creates a pin constraint for particle `i` at `target`.
    #[must_use]
    pub fn new(i: usize, target: [f32; 3]) -> Self {
        Self { i, target }
    }
}

impl Constraint for PinConstraint {
    fn num_particles(&self) -> usize {
        1
    }

    fn particle_index(&self, _k: usize) -> usize {
        self.i
    }

    fn solve(
        &self,
        particles: &[Particle],
        predicted: &[[f32; 3]],
        dx: &mut [[f32; 3]],
        n_affecting: &mut [u32],
    ) {
        if particles[self.i].inv_mass <= 0.0 {
            return;
        }
        let pi = predicted[self.i];
        let diff = [
            self.target[0] - pi[0],
            self.target[1] - pi[1],
            self.target[2] - pi[2],
        ];
        dx[self.i][0] += diff[0];
        dx[self.i][1] += diff[1];
        dx[self.i][2] += diff[2];
        n_affecting[self.i] = n_affecting[self.i].saturating_add(1);
    }
}

/// Half-space (plane) constraint: keeps a particle on or above a plane.
///
/// Constraint C = dot(normal, p - point) - radius ≥ 0. The plane normal should
/// point to the allowed side (e.g. `[0, 1, 0]` for ground so particles stay above y = 0).
/// When C < 0 the particle is pushed along +normal by the penetration depth.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PlaneConstraint {
    /// Index of the particle.
    pub particle_index: usize,
    /// A point on the plane.
    pub point_on_plane: [f32; 3],
    /// Unit normal pointing to the allowed side (e.g. up for ground).
    pub plane_normal: [f32; 3],
}

impl PlaneConstraint {
    /// Creates a plane constraint for particle `particle_index`.
    /// `plane_normal` should be unit length and point to the allowed side.
    #[must_use]
    pub fn new(particle_index: usize, point_on_plane: [f32; 3], plane_normal: [f32; 3]) -> Self {
        Self {
            particle_index,
            point_on_plane,
            plane_normal,
        }
    }
}

impl Constraint for PlaneConstraint {
    fn num_particles(&self) -> usize {
        1
    }

    fn particle_index(&self, k: usize) -> usize {
        if k == 0 {
            self.particle_index
        } else {
            self.particle_index
        }
    }

    fn solve(
        &self,
        particles: &[Particle],
        predicted: &[[f32; 3]],
        dx: &mut [[f32; 3]],
        n_affecting: &mut [u32],
    ) {
        let i = self.particle_index;
        if particles.get(i).map_or(true, |p| p.inv_mass <= 0.0) {
            return;
        }
        let p = predicted[i];
        let radius = particles[i].radius;
        let dx_pos = [
            p[0] - self.point_on_plane[0],
            p[1] - self.point_on_plane[1],
            p[2] - self.point_on_plane[2],
        ];
        let c = dx_pos[0] * self.plane_normal[0]
            + dx_pos[1] * self.plane_normal[1]
            + dx_pos[2] * self.plane_normal[2]
            - radius;
        if c >= 0.0 {
            return;
        }
        let correction = -c;
        dx[i][0] += correction * self.plane_normal[0];
        dx[i][1] += correction * self.plane_normal[1];
        dx[i][2] += correction * self.plane_normal[2];
        n_affecting[i] = n_affecting[i].saturating_add(1);
    }
}
