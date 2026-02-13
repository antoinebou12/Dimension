//! Shape-matching constraint for rigid bodies (Macklin et al. Sec 5).
//!
//! Computes covariance A = Σ(x*_i − c)·r_i^T, polar decomposition via SVD (Q = U V^T),
//! then Δx_i = (Q·r_i + c) − x*_i.

use crate::constraint::Constraint;
use crate::particle::Particle;
use mathlib::{Matrix, Storage, svd_econ};

/// Shape-matching constraint: keeps a set of particles in a rigid configuration.
///
/// Rest positions `rest` are relative to the body center. Each substep we compute
/// the best-fit rotation Q via SVD polar decomposition and apply Δx_i = (Q·r_i + c) − x*_i.
#[derive(Clone, Debug)]
pub struct ShapeMatchingConstraint {
    /// Particle indices (all in the same rigid body).
    pub indices: Vec<usize>,
    /// Rest positions relative to center (same length as indices).
    pub rest: Vec<[f32; 3]>,
}

impl ShapeMatchingConstraint {
    /// Creates a shape-matching constraint from particle indices and their rest positions (relative to center).
    ///
    /// # Panics
    /// Panics if `indices.len() != rest.len()` or if either is empty.
    #[must_use]
    pub fn new(indices: Vec<usize>, rest: Vec<[f32; 3]>) -> Self {
        assert_eq!(indices.len(), rest.len());
        assert!(!indices.is_empty());
        Self { indices, rest }
    }

    /// Builds rest positions from initial world positions (center = mean of positions).
    #[must_use]
    pub fn from_positions(indices: Vec<usize>, positions: &[[f32; 3]]) -> Self {
        assert_eq!(indices.len(), positions.len());
        assert!(!indices.is_empty());
        let n = indices.len() as f32;
        let mut c = [0.0f32; 3];
        for p in positions {
            c[0] += p[0];
            c[1] += p[1];
            c[2] += p[2];
        }
        c[0] /= n;
        c[1] /= n;
        c[2] /= n;
        let rest: Vec<[f32; 3]> = positions
            .iter()
            .map(|p| [p[0] - c[0], p[1] - c[1], p[2] - c[2]])
            .collect();
        Self::new(indices, rest)
    }
}

impl Constraint for ShapeMatchingConstraint {
    fn num_particles(&self) -> usize {
        self.indices.len()
    }

    fn particle_index(&self, k: usize) -> usize {
        self.indices[k]
    }

    fn solve(
        &self,
        particles: &[Particle],
        predicted: &[[f32; 3]],
        dx: &mut [[f32; 3]],
        n_affecting: &mut [u32],
    ) {
        let n = self.indices.len();
        if n == 0 {
            return;
        }
        let nf = n as f32;
        let mut c = [0.0f32; 3];
        for &i in &self.indices {
            if i < predicted.len() {
                c[0] += predicted[i][0];
                c[1] += predicted[i][1];
                c[2] += predicted[i][2];
            }
        }
        c[0] /= nf;
        c[1] /= nf;
        c[2] /= nf;

        let mut a = Matrix::with_storage(3, 3, Storage::Column);
        for i in 0..3 {
            for j in 0..3 {
                let mut sum = 0.0f64;
                for k in 0..n {
                    let idx = self.indices[k];
                    if idx >= predicted.len() {
                        continue;
                    }
                    let p = predicted[idx];
                    let r = self.rest[k];
                    sum += f64::from((p[i] - c[i]) * r[j]);
                }
                a.set(i, j, sum);
            }
        }

        let svd = svd_econ(&a);
        let u = svd.u();
        let v = svd.v();
        let mut q = Matrix::with_storage(3, 3, Storage::Column);
        for i in 0..3 {
            for j in 0..3 {
                let mut sum = 0.0f64;
                for k in 0..3 {
                    sum += u.get(i, k) * v.get(j, k);
                }
                q.set(i, j, sum);
            }
        }
        let det = q.get(0, 0) * (q.get(1, 1) * q.get(2, 2) - q.get(1, 2) * q.get(2, 1))
            - q.get(0, 1) * (q.get(1, 0) * q.get(2, 2) - q.get(1, 2) * q.get(2, 0))
            + q.get(0, 2) * (q.get(1, 0) * q.get(2, 1) - q.get(1, 1) * q.get(2, 0));
        if det < 0.0 {
            for i in 0..3 {
                q.set(i, 2, -q.get(i, 2));
            }
        }

        for (k, &idx) in self.indices.iter().enumerate() {
            if idx >= predicted.len() || idx >= particles.len() {
                continue;
            }
            if particles[idx].inv_mass <= 0.0 {
                continue;
            }
            let r = self.rest[k];
            let target = [
                (q.get(0, 0) * f64::from(r[0]) + q.get(0, 1) * f64::from(r[1]) + q.get(0, 2) * f64::from(r[2]) + f64::from(c[0])) as f32,
                (q.get(1, 0) * f64::from(r[0]) + q.get(1, 1) * f64::from(r[1]) + q.get(1, 2) * f64::from(r[2]) + f64::from(c[1])) as f32,
                (q.get(2, 0) * f64::from(r[0]) + q.get(2, 1) * f64::from(r[1]) + q.get(2, 2) * f64::from(r[2]) + f64::from(c[2])) as f32,
            ];
            let p = predicted[idx];
            dx[idx][0] += target[0] - p[0];
            dx[idx][1] += target[1] - p[1];
            dx[idx][2] += target[2] - p[2];
            n_affecting[idx] = n_affecting[idx].saturating_add(1);
        }
    }
}
