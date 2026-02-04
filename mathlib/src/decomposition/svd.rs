//! Singular Value Decomposition (SVD).
//!
//! SVD decomposes a matrix A (m×n) into: **A = U Σ V^T** where:
//! - **U** (m×m or m×k): Left singular vectors (orthonormal columns)
//! - **Σ** (diagonal): Singular values in descending order
//! - **V** (n×n or n×k): Right singular vectors (orthonormal columns)
//!
//! # Types
//!
//! - [`Svd`]: Full SVD with mutable decomposition via `decompose()`
//! - [`SvdEcon`]: Economical SVD via [`svd_econ`] (k = min(m, n) components)
//!
//! # Algorithm
//!
//! Uses the Golub-Reinsch algorithm with Householder bidiagonalization
//! followed by implicit QR iteration.
//!
//! # Example
//!
//! ```
//! use mathlib::{Matrix, svd_econ, Storage};
//!
//! let mut a = Matrix::<f64>::with_storage(3, 2, Storage::Column);
//! a.data_mut().copy_from_slice(&[1.0, 0.0, 0.0, 0.0, 2.0, 0.0]);
//! let svd = svd_econ(&a);
//! let u = svd.u();       // 3×2 matrix
//! let sigma = svd.sigma(); // 2-element vector
//! let v = svd.v();       // 2×2 matrix
//! ```
//!
//! # Applications
//!
//! - **Pseudoinverse**: Compute Moore-Penrose inverse
//! - **Low-rank approximation**: Truncate to k largest singular values
//! - **PCA**: Principal components are columns of V
//! - **Least squares**: Solve overdetermined systems

use crate::matrix::Matrix;
use crate::types::Storage;
use crate::vector::Vector;
use std::fmt;
use tracing::{debug, info};

fn sign_svd(a: f64, b: f64) -> f64 {
    if b > 0.0 { a.abs() } else { -a.abs() }
}

#[derive(Clone, Debug)]
pub struct Svd {
    u: Matrix<f64>,
    v: Matrix<f64>,
    sigma: Vector<f64>,
}

impl Svd {
    pub fn new(a: &Matrix<f64>) -> Self {
        let (nrows, ncols) = (a.rows(), a.cols());
        let mut u = Matrix::with_storage(nrows, ncols, Storage::Column);
        for i in 0..nrows {
            for j in 0..ncols {
                u.set(i, j, a.get(i, j));
            }
        }
        let mut v = Matrix::with_storage(ncols, ncols, Storage::Column);
        v.set_zero();
        for i in 0..ncols {
            v.set(i, i, 1.0);
        }
        let mut sigma = Vector::with_capacity(ncols);
        sigma.set_zero();
        for i in 0..ncols {
            sigma.set(i, 1.0);
        }
        Self { u, v, sigma }
    }

    /// Left singular vectors U.
    pub fn u(&self) -> &Matrix<f64> {
        &self.u
    }

    /// Right singular vectors V.
    pub fn v(&self) -> &Matrix<f64> {
        &self.v
    }

    /// Singular values sigma.
    pub fn sigma(&self) -> &Vector<f64> {
        &self.sigma
    }

    #[deprecated(since = "0.1.0", note = "use `u()` instead")]
    pub fn get_u(&self) -> &Matrix<f64> {
        self.u()
    }

    #[deprecated(since = "0.1.0", note = "use `v()` instead")]
    pub fn get_v(&self) -> &Matrix<f64> {
        self.v()
    }

    #[deprecated(since = "0.1.0", note = "use `sigma()` instead")]
    pub fn get_sigma(&self) -> &Vector<f64> {
        self.sigma()
    }

    #[allow(clippy::too_many_lines)]
    pub fn decompose(&mut self) {
        let ncols = self.u.cols();
        let nrows = self.u.rows();
        debug!(nrows, ncols, "svd decompose");
        let mut rv1 = Vector::with_capacity(ncols);
        rv1.set_zero();

        let mut g = 0.0f64;
        let mut scale = 0.0f64;
        let mut anorm = 0.0f64;

        for i in 0..ncols {
            let l = i + 1;
            rv1.set(i, scale * g);
            g = 0.0;
            scale = 0.0;
            if i < nrows {
                for k in i..nrows {
                    scale += self.u.get(k, i).abs();
                }
                if scale != 0.0 {
                    let mut s = 0.0;
                    for k in i..nrows {
                        let t = self.u.get(k, i) / scale;
                        self.u.set(k, i, t);
                        s += t * t;
                    }
                    let f = self.u.get(i, i);
                    g = -sign_svd(s.sqrt(), f);
                    let h = f * g - s;
                    self.u.set(i, i, f - g);
                    for j in l..ncols {
                        s = 0.0;
                        for k in i..nrows {
                            s += self.u.get(k, i) * self.u.get(k, j);
                        }
                        let f = s / h;
                        for k in i..nrows {
                            self.u.set(k, j, self.u.get(k, j) + f * self.u.get(k, i));
                        }
                    }
                    for k in i..nrows {
                        self.u.set(k, i, self.u.get(k, i) * scale);
                    }
                }
            }
            self.sigma.set(i, scale * g);
            g = 0.0;
            scale = 0.0;
            if i < nrows && i != ncols.saturating_sub(1) {
                for k in l..ncols {
                    scale += self.u.get(i, k).abs();
                }
                if scale != 0.0 {
                    let mut s = 0.0;
                    for k in l..ncols {
                        let t = self.u.get(i, k) / scale;
                        self.u.set(i, k, t);
                        s += t * t;
                    }
                    let f = self.u.get(i, l);
                    g = -sign_svd(s.sqrt(), f);
                    let h = f * g - s;
                    self.u.set(i, l, f - g);
                    for k in l..ncols {
                        rv1.set(k, self.u.get(i, k) / h);
                    }
                    for j in l..nrows {
                        s = 0.0;
                        for k in l..ncols {
                            s += self.u.get(j, k) * self.u.get(i, k);
                        }
                        for k in l..ncols {
                            self.u.set(j, k, self.u.get(j, k) + s * rv1.get(k));
                        }
                    }
                    for k in l..ncols {
                        self.u.set(i, k, self.u.get(i, k) * scale);
                    }
                }
            }
            let tmp = self.sigma.get(i).abs() + rv1.get(i).abs();
            anorm = anorm.max(tmp);
        }

        for i in (0..ncols).rev() {
            let l = i + 1;
            if i < ncols - 1 {
                if g != 0.0 {
                    for j in l..ncols {
                        self.v.set(j, i, (self.u.get(i, j) / self.u.get(i, l)) / g);
                    }
                    for j in l..ncols {
                        let mut s = 0.0;
                        for k in l..ncols {
                            s += self.u.get(i, k) * self.v.get(k, j);
                        }
                        for k in l..ncols {
                            self.v.set(k, j, self.v.get(k, j) + s * self.v.get(k, i));
                        }
                    }
                }
                for j in l..ncols {
                    self.v.set(i, j, 0.0);
                    self.v.set(j, i, 0.0);
                }
            }
            self.v.set(i, i, 1.0);
            g = rv1.get(i);
        }

        let min_rc = nrows.min(ncols);
        for i in (0..min_rc).rev() {
            let l = i + 1;
            g = self.sigma.get(i);
            for j in l..ncols {
                self.u.set(i, j, 0.0);
            }
            if g == 0.0 {
                for j in i..nrows {
                    self.u.set(j, i, 0.0);
                }
            } else {
                let g_inv = 1.0 / g;
                for j in l..ncols {
                    let mut s = 0.0;
                    for k in l..nrows {
                        s += self.u.get(k, i) * self.u.get(k, j);
                    }
                    let f = (s / self.u.get(i, i)) * g_inv;
                    for k in i..nrows {
                        self.u.set(k, j, self.u.get(k, j) + f * self.u.get(k, i));
                    }
                }
                for j in i..nrows {
                    self.u.set(j, i, self.u.get(j, i) * g_inv);
                }
            }
            self.u.set(i, i, self.u.get(i, i) + 1.0);
        }

        self.reorder();
        info!("svd decompose ok");
    }

    fn reorder(&mut self) {
        let ncols = self.u.cols();
        let nrows = self.u.rows();
        let mut inc = 1usize;
        while inc <= ncols {
            inc = inc * 3 + 1;
        }
        while inc > 1 {
            inc /= 3;
            for i in inc..ncols {
                let sw = self.sigma.get(i);
                let su: Vec<f64> = (0..nrows).map(|k| self.u.get(k, i)).collect();
                let sv: Vec<f64> = (0..ncols).map(|k| self.v.get(k, i)).collect();
                let mut j = i;
                while j >= inc && self.sigma.get(j - inc) < sw {
                    self.sigma.set(j, self.sigma.get(j - inc));
                    for k in 0..nrows {
                        self.u.set(k, j, self.u.get(k, j - inc));
                    }
                    for k in 0..ncols {
                        self.v.set(k, j, self.v.get(k, j - inc));
                    }
                    j -= inc;
                    if j < inc {
                        break;
                    }
                }
                self.sigma.set(j, sw);
                for (k, &v) in su.iter().enumerate() {
                    self.u.set(k, j, v);
                }
                for (k, &v) in sv.iter().enumerate() {
                    self.v.set(k, j, v);
                }
            }
        }
    }
}

/// Economical SVD: only the first k = min(m, n) singular values and corresponding columns of U and V.
#[derive(Clone, Debug)]
pub struct SvdEcon {
    u: Matrix<f64>,
    v: Matrix<f64>,
    sigma: Vector<f64>,
}

impl SvdEcon {
    /// U (m×k), V (n×k), sigma length k with k = min(m, n).
    pub fn u(&self) -> &Matrix<f64> {
        &self.u
    }

    pub fn v(&self) -> &Matrix<f64> {
        &self.v
    }

    pub fn sigma(&self) -> &Vector<f64> {
        &self.sigma
    }

    #[deprecated(since = "0.1.0", note = "use `u()` instead")]
    pub fn get_u(&self) -> &Matrix<f64> {
        self.u()
    }

    #[deprecated(since = "0.1.0", note = "use `v()` instead")]
    pub fn get_v(&self) -> &Matrix<f64> {
        self.v()
    }

    #[deprecated(since = "0.1.0", note = "use `sigma()` instead")]
    pub fn get_sigma(&self) -> &Vector<f64> {
        self.sigma()
    }
}

/// Economical singular value decomposition: returns U (m×k), V (n×k), sigma (length k) with k = min(m, n).
pub fn svd_econ(a: &Matrix<f64>) -> SvdEcon {
    let rows = a.rows();
    let cols = a.cols();
    debug!(rows, cols, "svd_econ");
    let mut full = Svd::new(a);
    full.decompose();
    let (m, n) = (full.u().rows(), full.v().rows());
    let k = m.min(n);
    let mut u = Matrix::with_storage(m, k, Storage::Column);
    let mut v = Matrix::with_storage(n, k, Storage::Column);
    let mut sigma = Vector::with_capacity(k);
    sigma.set_zero();
    for j in 0..k {
        sigma.set(j, full.sigma().get(j));
        for i in 0..m {
            u.set(i, j, full.u().get(i, j));
        }
        for i in 0..n {
            v.set(i, j, full.v().get(i, j));
        }
    }
    SvdEcon { u, v, sigma }
}

impl fmt::Display for Svd {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Svd U={}x{} V={}x{} sigma len={}",
            self.u.rows(),
            self.u.cols(),
            self.v.rows(),
            self.v.cols(),
            self.sigma.rows()
        )
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Svd {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("Svd", 3)?;
        s.serialize_field("u", &self.u)?;
        s.serialize_field("v", &self.v)?;
        s.serialize_field("sigma", &self.sigma)?;
        s.end()
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Svd {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct SvdDe {
            u: Matrix<f64>,
            v: Matrix<f64>,
            sigma: Vector<f64>,
        }
        let d = SvdDe::deserialize(deserializer)?;
        Ok(Svd {
            u: d.u,
            v: d.v,
            sigma: d.sigma,
        })
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for SvdEcon {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("SvdEcon", 3)?;
        s.serialize_field("u", &self.u)?;
        s.serialize_field("v", &self.v)?;
        s.serialize_field("sigma", &self.sigma)?;
        s.end()
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for SvdEcon {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct SvdEconDe {
            u: Matrix<f64>,
            v: Matrix<f64>,
            sigma: Vector<f64>,
        }
        let d = SvdEconDe::deserialize(deserializer)?;
        Ok(SvdEcon {
            u: d.u,
            v: d.v,
            sigma: d.sigma,
        })
    }
}
