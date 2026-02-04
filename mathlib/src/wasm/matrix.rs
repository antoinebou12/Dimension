//! WasmMatrix, WasmMatrix32, and WasmSvd for JavaScript.

use wasm_bindgen::prelude::*;

use crate::decomposition::svd::svd_econ;
use crate::math3d::{matrix4_mul_vector3, matrix4f_inverse, transform_vector};
use crate::{Matrix, Storage, Vector, solve};

use super::vector::WasmVector;

/// A dense matrix accessible from JavaScript.
#[wasm_bindgen]
pub struct WasmMatrix {
    pub(crate) inner: Matrix<f64>,
}

#[wasm_bindgen]
impl WasmMatrix {
    /// Create a new zero matrix with given dimensions (column-major).
    #[wasm_bindgen(constructor)]
    pub fn new(rows: usize, cols: usize) -> Self {
        Self {
            inner: Matrix::with_storage(rows, cols, Storage::Column),
        }
    }

    /// Create a matrix from a flat array (column-major order).
    #[wasm_bindgen(js_name = fromArray)]
    pub fn from_array(rows: usize, cols: usize, data: &[f64]) -> Result<WasmMatrix, JsError> {
        if data.len() != rows * cols {
            return Err(JsError::new(&format!(
                "Data length {} does not match dimensions {}x{}",
                data.len(),
                rows,
                cols
            )));
        }
        let mut m = Matrix::with_storage(rows, cols, Storage::Column);
        for (i, &val) in data.iter().enumerate() {
            m.data_mut()[i] = val;
        }
        Ok(Self { inner: m })
    }

    /// Create an identity matrix.
    #[wasm_bindgen(js_name = identity)]
    pub fn identity(n: usize) -> Self {
        let mut m = Matrix::with_storage(n, n, Storage::Column);
        m.set_identity();
        Self { inner: m }
    }

    /// Get the number of rows.
    #[wasm_bindgen(getter)]
    pub fn rows(&self) -> usize {
        self.inner.rows()
    }

    /// Get the number of columns.
    #[wasm_bindgen(getter)]
    pub fn cols(&self) -> usize {
        self.inner.cols()
    }

    /// Get element at (i, j).
    pub fn get(&self, i: usize, j: usize) -> f64 {
        self.inner.get(i, j)
    }

    /// Set element at (i, j).
    pub fn set(&mut self, i: usize, j: usize, value: f64) {
        self.inner.set(i, j, value);
    }

    /// Return data as a flat Float64Array (column-major).
    #[wasm_bindgen(js_name = toArray)]
    pub fn to_array(&self) -> Vec<f64> {
        self.inner.data().to_vec()
    }

    /// Transpose the matrix (returns new matrix).
    pub fn transpose(&self) -> WasmMatrix {
        Self {
            inner: self.inner.transpose(),
        }
    }

    /// Matrix addition.
    pub fn add(&self, other: &WasmMatrix) -> Result<WasmMatrix, JsError> {
        if self.rows() != other.rows() || self.cols() != other.cols() {
            return Err(JsError::new("Matrix dimensions must match for addition"));
        }
        Ok(Self {
            inner: &self.inner + &other.inner,
        })
    }

    /// Matrix subtraction.
    pub fn sub(&self, other: &WasmMatrix) -> Result<WasmMatrix, JsError> {
        if self.rows() != other.rows() || self.cols() != other.cols() {
            return Err(JsError::new("Matrix dimensions must match for subtraction"));
        }
        Ok(Self {
            inner: &self.inner - &other.inner,
        })
    }

    /// Matrix multiplication.
    #[wasm_bindgen(js_name = mul)]
    pub fn mul(&self, other: &WasmMatrix) -> Result<WasmMatrix, JsError> {
        if self.cols() != other.rows() {
            return Err(JsError::new(&format!(
                "Cannot multiply {}x{} by {}x{}",
                self.rows(),
                self.cols(),
                other.rows(),
                other.cols()
            )));
        }
        Ok(Self {
            inner: &self.inner * &other.inner,
        })
    }

    /// Scalar multiplication.
    #[wasm_bindgen(js_name = scale)]
    pub fn scale(&self, scalar: f64) -> WasmMatrix {
        let mut result = Matrix::with_storage(self.rows(), self.cols(), Storage::Column);
        for i in 0..self.inner.data().len() {
            result.data_mut()[i] = self.inner.data()[i] * scalar;
        }
        Self { inner: result }
    }

    /// Matrix-vector multiplication.
    #[wasm_bindgen(js_name = mulVector)]
    pub fn mul_vector(&self, v: &WasmVector) -> Result<WasmVector, JsError> {
        if self.cols() != v.len() {
            return Err(JsError::new(&format!(
                "Cannot multiply {}x{} matrix by vector of length {}",
                self.rows(),
                self.cols(),
                v.len()
            )));
        }
        Ok(WasmVector {
            inner: &self.inner * &v.inner,
        })
    }

    /// Solve Ax = b for square matrix A. Returns x or an error if A is singular or not square.
    #[wasm_bindgen(js_name = solve)]
    pub fn solve(&self, b: &WasmVector) -> Result<WasmVector, JsError> {
        let x = solve(&self.inner, &b.inner).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmVector { inner: x })
    }

    /// Economical SVD: returns U, V, and singular values sigma (min(m,n) components).
    #[wasm_bindgen(js_name = svdEcon)]
    pub fn svd_econ(&self) -> WasmSvd {
        let econ = svd_econ(&self.inner);
        WasmSvd {
            u: econ.u().clone(),
            v: econ.v().clone(),
            sigma: econ.sigma().clone(),
        }
    }
}

/// Result of economical SVD: U (m×k), V (n×k), sigma (length k) with k = min(m, n).
#[wasm_bindgen]
pub struct WasmSvd {
    u: Matrix<f64>,
    v: Matrix<f64>,
    sigma: Vector<f64>,
}

#[wasm_bindgen]
impl WasmSvd {
    /// Left singular vectors U (matrix).
    #[wasm_bindgen(js_name = getU)]
    pub fn get_u(&self) -> WasmMatrix {
        WasmMatrix {
            inner: self.u.clone(),
        }
    }

    /// Right singular vectors V (matrix).
    #[wasm_bindgen(js_name = getV)]
    pub fn get_v(&self) -> WasmMatrix {
        WasmMatrix {
            inner: self.v.clone(),
        }
    }

    /// Singular values (vector).
    #[wasm_bindgen(js_name = getSigma)]
    pub fn get_sigma(&self) -> WasmVector {
        WasmVector {
            inner: self.sigma.clone(),
        }
    }
}

/// A 32-bit float matrix for 3D graphics operations.
#[wasm_bindgen]
pub struct WasmMatrix32 {
    inner: Matrix<f32>,
}

impl WasmMatrix32 {
    /// Build from inner matrix (for use by wasm submodules).
    pub(crate) fn from_inner(inner: Matrix<f32>) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen]
impl WasmMatrix32 {
    /// Create a new zero matrix with given dimensions.
    #[wasm_bindgen(constructor)]
    pub fn new(rows: usize, cols: usize) -> Self {
        Self {
            inner: Matrix::with_storage(rows, cols, Storage::Column),
        }
    }

    /// Create a 4x4 identity matrix.
    #[wasm_bindgen(js_name = identity4)]
    pub fn identity4() -> Self {
        let mut m = Matrix::with_storage(4, 4, Storage::Column);
        m.set_identity();
        Self { inner: m }
    }

    /// Create a 4x4 rotation matrix from Euler angles (radians).
    #[wasm_bindgen(js_name = rotation)]
    pub fn rotation(rx: f32, ry: f32, rz: f32) -> Self {
        use crate::make_rotation;
        let r3 = make_rotation(rx, ry, rz);
        let mut m = Matrix::with_storage(4, 4, Storage::Column);
        m.set_identity();
        for i in 0..3 {
            for j in 0..3 {
                m.set(i, j, r3.get(i, j));
            }
        }
        Self { inner: m }
    }

    /// Create a matrix from a flat array (column-major order).
    #[wasm_bindgen(js_name = fromArray)]
    pub fn from_array(rows: usize, cols: usize, data: &[f32]) -> Result<WasmMatrix32, JsError> {
        if data.len() != rows * cols {
            return Err(JsError::new(&format!(
                "Data length {} does not match dimensions {}x{}",
                data.len(),
                rows,
                cols
            )));
        }
        let mut m = Matrix::with_storage(rows, cols, Storage::Column);
        for (i, &val) in data.iter().enumerate() {
            m.data_mut()[i] = val;
        }
        Ok(Self { inner: m })
    }

    /// Get the number of rows.
    #[wasm_bindgen(getter)]
    pub fn rows(&self) -> usize {
        self.inner.rows()
    }

    /// Get the number of columns.
    #[wasm_bindgen(getter)]
    pub fn cols(&self) -> usize {
        self.inner.cols()
    }

    /// Get element at (i, j).
    pub fn get(&self, i: usize, j: usize) -> f32 {
        self.inner.get(i, j)
    }

    /// Set element at (i, j).
    pub fn set(&mut self, i: usize, j: usize, value: f32) {
        self.inner.set(i, j, value);
    }

    /// Return data as a flat array.
    #[wasm_bindgen(js_name = toArray)]
    pub fn to_array(&self) -> Vec<f32> {
        self.inner.data().to_vec()
    }

    /// Transpose (returns new matrix).
    pub fn transpose(&self) -> WasmMatrix32 {
        Self {
            inner: self.inner.transpose(),
        }
    }

    /// Inverse for 4×4 matrices (e.g. view/model). Errors if not 4×4.
    pub fn inverse(&self) -> Result<WasmMatrix32, JsError> {
        if self.rows() != 4 || self.cols() != 4 {
            return Err(JsError::new("inverse() only supported for 4×4 matrices"));
        }
        Ok(Self {
            inner: matrix4f_inverse(&self.inner),
        })
    }

    /// Transform a 3D point (x, y, z) by this 4×4 matrix. Returns [x', y', z'].
    #[wasm_bindgen(js_name = transformPoint)]
    pub fn transform_point(&self, x: f32, y: f32, z: f32) -> Result<Vec<f32>, JsError> {
        if self.rows() != 4 || self.cols() != 4 {
            return Err(JsError::new("transformPoint requires a 4×4 matrix"));
        }
        let v = crate::vector3(x, y, z);
        let out = matrix4_mul_vector3(&self.inner, &v);
        Ok(vec![out.get(0), out.get(1), out.get(2)])
    }

    /// Transform a 3D direction (x, y, z) by the 3×3 part only (no translation).
    #[wasm_bindgen(js_name = transformVector)]
    pub fn transform_vector(&self, x: f32, y: f32, z: f32) -> Result<Vec<f32>, JsError> {
        if self.rows() != 4 || self.cols() != 4 {
            return Err(JsError::new("transformVector requires a 4×4 matrix"));
        }
        let v = crate::vector3(x, y, z);
        let out = transform_vector(&self.inner, &v);
        Ok(vec![out.get(0), out.get(1), out.get(2)])
    }

    /// Matrix multiplication.
    #[wasm_bindgen(js_name = mul)]
    pub fn mul(&self, other: &WasmMatrix32) -> Result<WasmMatrix32, JsError> {
        if self.cols() != other.rows() {
            return Err(JsError::new(&format!(
                "Cannot multiply {}x{} by {}x{}",
                self.rows(),
                self.cols(),
                other.rows(),
                other.cols()
            )));
        }
        Ok(Self {
            inner: &self.inner * &other.inner,
        })
    }

    /// Matrix multiplication on CPU only (for fair CPU vs GPU comparison in demos).
    /// Computes C = A × B using a triple loop; does not use the GPU backend.
    #[wasm_bindgen(js_name = matmulF32Cpu)]
    pub fn matmul_f32_cpu(&self, other: &WasmMatrix32) -> Result<WasmMatrix32, JsError> {
        if self.cols() != other.rows() {
            return Err(JsError::new(&format!(
                "Cannot multiply {}x{} by {}x{}",
                self.rows(),
                self.cols(),
                other.rows(),
                other.cols()
            )));
        }
        let a = &self.inner;
        let b = &other.inner;
        let m = a.rows();
        let k = a.cols();
        let n = b.cols();
        let mut out = Matrix::with_storage(m, n, Storage::Column);
        out.set_zero();
        for i in 0..m {
            for j in 0..n {
                let mut sum = 0f32;
                for kk in 0..k {
                    sum += a.get(i, kk) * b.get(kk, j);
                }
                out.set(i, j, sum);
            }
        }
        Ok(Self { inner: out })
    }
}
