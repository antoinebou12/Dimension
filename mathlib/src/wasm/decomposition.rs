//! WasmPca, WasmCholesky, and WasmLu for JavaScript.

use wasm_bindgen::prelude::*;

use crate::lu::Lu;
use crate::{Cholesky, Matrix, Storage, Vector};

use super::matrix::WasmMatrix;
#[cfg(feature = "gpu")]
use super::matrix::WasmMatrix32;
use super::vector::WasmVector;

/// Result of PCA: mean, components, and explained variance.
#[wasm_bindgen]
pub struct WasmPca {
    mean: Vector<f64>,
    components: Matrix<f64>,
    explained_variance: Vector<f64>,
}

#[wasm_bindgen]
impl WasmPca {
    /// Run PCA on data matrix (rows = samples, cols = features).
    /// Returns mean, principal components, and explained variance.
    /// If `n_components` is 0, all components are kept.
    #[wasm_bindgen(constructor)]
    pub fn new(data: &WasmMatrix, n_components: usize) -> Self {
        let n = if n_components == 0 {
            None
        } else {
            Some(n_components)
        };
        let result = crate::pca(&data.inner, n);
        Self {
            mean: result.mean().clone(),
            components: result.components().clone(),
            explained_variance: result.explained_variance().clone(),
        }
    }

    /// Mean vector (one per feature).
    #[wasm_bindgen(js_name = getMean)]
    pub fn get_mean(&self) -> WasmVector {
        WasmVector {
            inner: self.mean.clone(),
        }
    }

    /// Principal components matrix (features × components); each column is a PC.
    #[wasm_bindgen(js_name = getComponents)]
    pub fn get_components(&self) -> WasmMatrix {
        WasmMatrix {
            inner: self.components.clone(),
        }
    }

    /// Explained variance for each component.
    #[wasm_bindgen(js_name = getExplainedVariance)]
    pub fn get_explained_variance(&self) -> WasmVector {
        WasmVector {
            inner: self.explained_variance.clone(),
        }
    }

    /// Number of components.
    #[wasm_bindgen(js_name = nComponents)]
    pub fn n_components(&self) -> usize {
        self.components.cols()
    }

    /// Project data onto principal components. Returns projected matrix (samples × components).
    #[wasm_bindgen(js_name = transform)]
    pub fn transform(&self, data: &WasmMatrix) -> Result<WasmMatrix, JsError> {
        let (n_samples, n_features) = (data.inner.rows(), data.inner.cols());
        if n_features != self.mean.rows() {
            return Err(JsError::new(&format!(
                "Data has {} features but PCA expects {}",
                n_features,
                self.mean.rows()
            )));
        }
        let n_comp = self.components.cols();
        let mut result = Matrix::with_storage(n_samples, n_comp, Storage::Column);
        for i in 0..n_samples {
            for j in 0..n_comp {
                let mut sum = 0.0;
                for k in 0..n_features {
                    let centered = data.inner.get(i, k) - self.mean.get(k);
                    sum += centered * self.components.get(k, j);
                }
                result.set(i, j, sum);
            }
        }
        Ok(WasmMatrix { inner: result })
    }

    /// Project f32 data onto principal components using GPU matmul. Returns a Promise that resolves
    /// to the projected matrix (samples × components) or null if GPU is not available or fails.
    /// Call after initGpuAsync(). Falls back to sync transform() with f64 data if result is null.
    #[cfg(feature = "gpu")]
    #[wasm_bindgen(js_name = transformF32GpuAsync)]
    pub fn transform_f32_gpu_async(&self, data: &WasmMatrix32) -> js_sys::Promise {
        let n_samples = data.rows();
        let n_features = data.cols();
        if n_features != self.mean.rows() || self.components.rows() != self.mean.rows() {
            return js_sys::Promise::reject(
                &JsError::new(&format!(
                    "Data has {} features but PCA expects {}",
                    n_features,
                    self.mean.rows()
                ))
                .into(),
            );
        }
        let n_comp = self.components.cols();
        let mean_f32: Vec<f32> = (0..self.mean.rows())
            .map(|j| self.mean.get(j) as f32)
            .collect();
        let mut components_f32 =
            Matrix::with_storage(self.components.rows(), n_comp, Storage::Column);
        for i in 0..self.components.rows() {
            for j in 0..n_comp {
                components_f32.set(i, j, self.components.get(i, j) as f32);
            }
        }
        let data_inner = data.clone_inner();
        wasm_bindgen_futures::future_to_promise(async move {
            let mut centered = Matrix::with_storage(n_samples, n_features, Storage::Column);
            for i in 0..n_samples {
                for j in 0..n_features {
                    centered.set(i, j, data_inner.get(i, j) - mean_f32[j]);
                }
            }
            let result = crate::gpu::try_matmul_f32_async(&centered, &components_f32).await;
            Ok(match result {
                Some(m) => JsValue::from(WasmMatrix32::from_inner(m)),
                None => JsValue::NULL,
            })
        })
    }
}

/// Cholesky decomposition: A = L L^T for symmetric positive definite A.
#[wasm_bindgen]
pub struct WasmCholesky {
    factor: Matrix<f64>,
}

#[wasm_bindgen]
impl WasmCholesky {
    /// Compute Cholesky decomposition of matrix A. A must be square and symmetric positive definite.
    #[wasm_bindgen(constructor)]
    pub fn new(a: &WasmMatrix) -> Result<WasmCholesky, JsError> {
        let chol = Cholesky::new(&a.inner).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(Self {
            factor: chol.l().clone(),
        })
    }

    /// Lower triangular factor L (A = L L^T).
    #[wasm_bindgen(js_name = getL)]
    pub fn get_l(&self) -> WasmMatrix {
        WasmMatrix {
            inner: self.factor.clone(),
        }
    }

    /// Solve Ax = b where A = L L^T. Returns x.
    #[wasm_bindgen(js_name = solve)]
    pub fn solve(&self, b: &WasmVector) -> Result<WasmVector, JsError> {
        let n = self.factor.rows();
        if b.len() != n {
            return Err(JsError::new(&format!(
                "Vector length {} does not match matrix size {}",
                b.len(),
                n
            )));
        }
        let mut y = Vector::with_capacity(n);
        for i in 0..n {
            let mut sum = b.inner.get(i);
            for j in 0..i {
                sum -= self.factor.get(i, j) * y.get(j);
            }
            y.set(i, sum / self.factor.get(i, i));
        }
        let mut x = Vector::with_capacity(n);
        for i in (0..n).rev() {
            let mut sum = y.get(i);
            for j in (i + 1)..n {
                sum -= self.factor.get(j, i) * x.get(j);
            }
            x.set(i, sum / self.factor.get(i, i));
        }
        Ok(WasmVector { inner: x })
    }
}

/// LU decomposition: P A = L U for general square A.
#[wasm_bindgen]
pub struct WasmLu {
    factor: Matrix<f64>,
    pivot: Vec<usize>,
    sign: i8,
}

#[wasm_bindgen]
impl WasmLu {
    /// Compute LU decomposition of matrix A. A must be square and non-singular.
    #[wasm_bindgen(constructor)]
    pub fn new(a: &WasmMatrix) -> Result<WasmLu, JsError> {
        let lu = Lu::new(&a.inner).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(Self {
            factor: lu.lu().clone(),
            pivot: lu.pivot().to_vec(),
            sign: lu.sign(),
        })
    }

    /// Combined LU factor (L unit lower, U upper).
    #[wasm_bindgen(js_name = getLU)]
    pub fn get_lu(&self) -> WasmMatrix {
        WasmMatrix {
            inner: self.factor.clone(),
        }
    }

    /// Determinant of the original matrix.
    #[wasm_bindgen(js_name = determinant)]
    pub fn det(&self) -> f64 {
        let n = self.factor.rows();
        let sign = f64::from(self.sign);
        let mut prod = sign;
        for i in 0..n {
            prod *= self.factor.get(i, i);
        }
        prod
    }

    /// Solve Ax = b. Returns x.
    #[wasm_bindgen(js_name = solve)]
    pub fn solve(&self, b: &WasmVector) -> Result<WasmVector, JsError> {
        let n = self.factor.rows();
        if b.len() != n {
            return Err(JsError::new(&format!(
                "Vector length {} does not match matrix size {}",
                b.len(),
                n
            )));
        }
        let mut pb = Vector::with_capacity(n);
        for i in 0..n {
            pb.set(i, b.inner.get(self.pivot[i]));
        }
        let mut y = Vector::with_capacity(n);
        for i in 0..n {
            let mut s = pb.get(i);
            for j in 0..i {
                s -= self.factor.get(i, j) * y.get(j);
            }
            y.set(i, s);
        }
        let mut x = Vector::with_capacity(n);
        for i in (0..n).rev() {
            let mut s = y.get(i);
            for j in (i + 1)..n {
                s -= self.factor.get(i, j) * x.get(j);
            }
            let uii = self.factor.get(i, i);
            if uii.abs() < 1e-15 {
                return Err(JsError::new("LU factor is singular"));
            }
            x.set(i, s / uii);
        }
        Ok(WasmVector { inner: x })
    }
}
