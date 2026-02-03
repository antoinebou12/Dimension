//! Distance metrics: Euclidean, Manhattan, Minkowski, cosine, Chebyshev.
//!
//! For contiguous slices (e.g. two `Vector<f64>`), Euclidean/squared use the cpu backends
//! (SIMD > parallel > sequential) when features are enabled. Matrix row-row distances use a scalar loop.

use crate::matrix::Matrix;
use crate::vector::Vector;

/// Sum of squared differences for contiguous slices: `sum_i` (a[i] - b[i])^2.
/// Dispatches to sequential/simd/parallel backend like `Vector::dot_f64`.
#[inline]
fn squared_diff_sum_f64_slices(a: &[f64], b: &[f64]) -> f64 {
    #[cfg(feature = "simd")]
    return crate::cpu::simd::squared_diff_sum_f64(a, b);
    #[cfg(all(
        feature = "parallel",
        not(target_arch = "wasm32"),
        not(feature = "simd")
    ))]
    return crate::cpu::parallel::par_squared_diff_sum_f64(a, b);
    #[cfg(not(any(
        feature = "simd",
        all(feature = "parallel", not(target_arch = "wasm32"))
    )))]
    return crate::cpu::sequential::squared_diff_sum_f64(a, b);
}

// --- Euclidean ---

/// Squared Euclidean distance between two vectors (contiguous slices); uses cpu backend when enabled.
#[must_use]
pub fn squared_euclidean(a: &Vector<f64>, b: &Vector<f64>) -> f64 {
    assert_eq!(a.rows(), b.rows());
    squared_diff_sum_f64_slices(a.data(), b.data())
}

/// Euclidean distance between two vectors.
#[must_use]
pub fn euclidean(a: &Vector<f64>, b: &Vector<f64>) -> f64 {
    squared_euclidean(a, b).sqrt()
}

/// Squared Euclidean distance between row `i` and row `j` of matrix `data` (rows = samples, cols = features).
#[must_use]
pub fn squared_euclidean_rows(data: &Matrix<f64>, i: usize, j: usize) -> f64 {
    let n = data.cols();
    let mut sum = 0.0;
    for c in 0..n {
        let d = data.get(i, c) - data.get(j, c);
        sum += d * d;
    }
    sum
}

/// Euclidean distance between row `i` and row `j` of matrix `data`.
#[must_use]
pub fn euclidean_rows(data: &Matrix<f64>, i: usize, j: usize) -> f64 {
    squared_euclidean_rows(data, i, j).sqrt()
}

// --- Manhattan (L1) ---

/// Manhattan distance between two vectors: `sum_i` |a[i] - b[i]|.
#[must_use]
pub fn manhattan(a: &Vector<f64>, b: &Vector<f64>) -> f64 {
    assert_eq!(a.rows(), b.rows());
    let n = a.rows();
    let mut sum = 0.0;
    for i in 0..n {
        sum += (a.get(i) - b.get(i)).abs();
    }
    sum
}

/// Manhattan distance between row `i` and row `j` of matrix `data`.
#[must_use]
pub fn manhattan_rows(data: &Matrix<f64>, i: usize, j: usize) -> f64 {
    let n = data.cols();
    let mut sum = 0.0;
    for c in 0..n {
        sum += (data.get(i, c) - data.get(j, c)).abs();
    }
    sum
}

// --- Minkowski ---

/// Minkowski distance with exponent `p`: (`sum_i` |a[i] - b[i]|^p)^(1/p). Use `f64::INFINITY` for Chebyshev.
#[must_use]
pub fn minkowski(a: &Vector<f64>, b: &Vector<f64>, p: f64) -> f64 {
    assert_eq!(a.rows(), b.rows());
    if p == f64::INFINITY {
        return chebyshev(a, b);
    }
    let n = a.rows();
    let mut sum = 0.0;
    for i in 0..n {
        sum += (a.get(i) - b.get(i)).abs().powf(p);
    }
    sum.powf(1.0 / p)
}

/// Minkowski distance between row `i` and row `j` of matrix `data`.
#[must_use]
pub fn minkowski_rows(data: &Matrix<f64>, i: usize, j: usize, p: f64) -> f64 {
    if p == f64::INFINITY {
        return chebyshev_rows(data, i, j);
    }
    let n = data.cols();
    let mut sum = 0.0;
    for c in 0..n {
        sum += (data.get(i, c) - data.get(j, c)).abs().powf(p);
    }
    sum.powf(1.0 / p)
}

// --- Cosine ---

/// Cosine similarity: dot(a,b) / (norm(a) * norm(b)). Returns value in [-1, 1].
#[must_use]
pub fn cosine_similarity(a: &Vector<f64>, b: &Vector<f64>) -> f64 {
    assert_eq!(a.rows(), b.rows());
    let dot = a.dot(b);
    let na = a.norm();
    let nb = b.norm();
    if na < 1e-20 || nb < 1e-20 {
        return 0.0;
    }
    (dot / (na * nb)).clamp(-1.0, 1.0)
}

/// Cosine distance: 1 - `cosine_similarity` (so 0 = same direction, 2 = opposite).
#[must_use]
pub fn cosine_distance(a: &Vector<f64>, b: &Vector<f64>) -> f64 {
    1.0 - cosine_similarity(a, b)
}

/// Cosine similarity between row `i` and row `j` of matrix `data`.
#[must_use]
pub fn cosine_similarity_rows(data: &Matrix<f64>, i: usize, j: usize) -> f64 {
    let n = data.cols();
    let mut dot = 0.0;
    let mut na_sq = 0.0;
    let mut b_sq_sum = 0.0;
    for c in 0..n {
        let ai = data.get(i, c);
        let bj = data.get(j, c);
        dot += ai * bj;
        na_sq += ai * ai;
        b_sq_sum += bj * bj;
    }
    let na = na_sq.sqrt();
    let nb = b_sq_sum.sqrt();
    if na < 1e-20 || nb < 1e-20 {
        return 0.0;
    }
    (dot / (na * nb)).clamp(-1.0, 1.0)
}

/// Cosine distance between row `i` and row `j` of matrix `data`.
#[must_use]
pub fn cosine_distance_rows(data: &Matrix<f64>, i: usize, j: usize) -> f64 {
    1.0 - cosine_similarity_rows(data, i, j)
}

// --- Chebyshev (L_infinity) ---

/// Chebyshev distance: `max_i` |a[i] - b[i]|.
#[must_use]
pub fn chebyshev(a: &Vector<f64>, b: &Vector<f64>) -> f64 {
    assert_eq!(a.rows(), b.rows());
    let n = a.rows();
    let mut max_d = 0.0_f64;
    for i in 0..n {
        let d = (a.get(i) - b.get(i)).abs();
        if d > max_d {
            max_d = d;
        }
    }
    max_d
}

/// Chebyshev distance between row `i` and row `j` of matrix `data`.
#[must_use]
pub fn chebyshev_rows(data: &Matrix<f64>, i: usize, j: usize) -> f64 {
    let n = data.cols();
    let mut max_d = 0.0_f64;
    for c in 0..n {
        let d = (data.get(i, c) - data.get(j, c)).abs();
        if d > max_d {
            max_d = d;
        }
    }
    max_d
}
