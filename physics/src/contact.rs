//! Contact constraints: non-penetration (C = dist − r ≥ 0) and optional position-level friction.
//!
//! Uses the collision crate for shape bounds; contact constraints are generated from
//! broad-phase candidate pairs and solved in the PBD loop.

use crate::constraint::Constraint;
use crate::particle::Particle;

/// Non-penetration contact between two particles (spheres).
///
/// Constraint C = dist(x_i, x_j) − (r_i + r_j) ≥ 0. Only applies correction when penetrating.
#[derive(Clone, Debug)]
pub struct ContactConstraint {
    /// First particle index.
    pub i: usize,
    /// Second particle index.
    pub j: usize,
}

impl ContactConstraint {
    /// Creates a contact constraint between particles `i` and `j`.
    #[must_use]
    pub fn new(i: usize, j: usize) -> Self {
        Self { i, j }
    }
}

impl Constraint for ContactConstraint {
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
        particles: &[Particle],
        predicted: &[[f32; 3]],
        dx: &mut [[f32; 3]],
        n_affecting: &mut [u32],
    ) {
        let rest = particles[self.i].radius + particles[self.j].radius;
        if rest <= 0.0 {
            return;
        }
        let pi = predicted[self.i];
        let pj = predicted[self.j];
        let dx_ij = [pi[0] - pj[0], pi[1] - pj[1], pi[2] - pj[2]];
        let dist_sq = dx_ij[0] * dx_ij[0] + dx_ij[1] * dx_ij[1] + dx_ij[2] * dx_ij[2];
        let dist = dist_sq.sqrt().max(1e-8);
        let diff = dist - rest;
        if diff >= 0.0 {
            return;
        }
        let n_inv = 1.0 / dist;
        let nx = dx_ij[0] * n_inv;
        let ny = dx_ij[1] * n_inv;
        let nz = dx_ij[2] * n_inv;

        let wi = particles[self.i].inv_mass;
        let wj = particles[self.j].inv_mass;
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
