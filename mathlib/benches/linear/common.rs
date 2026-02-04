//! Shared test matrix generators for benchmarks.

#![allow(dead_code)]

use mathlib::{Triplet, Vector};

/// Generate a diagonal matrix as triplets.
pub fn generate_diagonal_matrix(n: usize) -> Vec<Triplet<f64>> {
    let mut triplets = Vec::new();
    for i in 0..n {
        triplets.push(Triplet::new((i + 1) as f64, i as u32, i as u32));
    }
    triplets
}

/// Generate a banded matrix as triplets.
pub fn generate_banded_matrix(n: usize, bandwidth: usize) -> Vec<Triplet<f64>> {
    let mut triplets = Vec::new();
    for i in 0..n {
        for j in i.saturating_sub(bandwidth)..=(i + bandwidth).min(n - 1) {
            let val = 1.0 + (i as f64) * 0.1 + (j as f64) * 0.01;
            triplets.push(Triplet::new(val, i as u32, j as u32));
        }
    }
    triplets
}

/// Generate a pseudo-random sparse matrix as triplets.
pub fn generate_random_sparse(n: usize, density: f64) -> Vec<Triplet<f64>> {
    let mut triplets = Vec::new();
    let nnz = ((n * n) as f64 * density) as usize;
    for k in 0..nnz {
        let i = (k * 7) % n;
        let j = (k * 13) % n;
        let val = (k as f64 + 1.0) * 0.1;
        triplets.push(Triplet::new(val, i as u32, j as u32));
    }
    triplets
}

/// Generate a small dense-ish matrix as triplets.
pub fn generate_small_dense(n: usize, density: f64) -> Vec<Triplet<f64>> {
    let mut triplets = Vec::new();
    let nnz = ((n * n) as f64 * density) as usize;
    for k in 0..nnz {
        let i = k / n;
        let j = k % n;
        if i < n && j < n {
            let val = (i + j + 1) as f64;
            triplets.push(Triplet::new(val, i as u32, j as u32));
        }
    }
    triplets
}

/// Generate a test vector of given size.
pub fn generate_vector(n: usize) -> Vector<f64> {
    let mut v = Vector::with_capacity(n);
    for i in 0..n {
        v.set(i, (i + 1) as f64);
    }
    v
}
