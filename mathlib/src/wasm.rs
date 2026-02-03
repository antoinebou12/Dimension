//! WebAssembly bindings for mathlib.
//!
//! Enable with `--features wasm`. Build with:
//! ```bash
//! wasm-pack build --target web --features wasm
//! ```
//!
//! Exposes dense matrices (f64/f32), vectors, linear solve Ax = b, SVD, 3D/camera helpers,
//! and easing (lerp, distance) for use from JavaScript.

use wasm_bindgen::prelude::*;

use crate::cg::{look_at_lh, look_at_rh, new_orthographic, new_perspective, new_translation};
use crate::decomposition::svd::svd_econ;
use crate::distance::euclidean;
use crate::math3d::{matrix4_mul_vector3, matrix4f_inverse, transform_vector};
use crate::{Matrix, Storage, Vector, solve};

// ============================================================================
// WasmMatrix — Dense matrix wrapper for JavaScript
// ============================================================================

/// A dense matrix accessible from JavaScript.
#[wasm_bindgen]
pub struct WasmMatrix {
    inner: Matrix<f64>,
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

// ============================================================================
// WasmVector — Vector wrapper for JavaScript
// ============================================================================

/// A vector accessible from JavaScript.
#[wasm_bindgen]
pub struct WasmVector {
    inner: Vector<f64>,
}

#[wasm_bindgen]
impl WasmVector {
    /// Create a new zero vector with given length.
    #[wasm_bindgen(constructor)]
    pub fn new(len: usize) -> Self {
        Self {
            inner: Vector::with_capacity(len),
        }
    }

    /// Create a vector from a Float64Array.
    #[wasm_bindgen(js_name = fromArray)]
    pub fn from_array(data: &[f64]) -> Self {
        let mut v = Vector::with_capacity(data.len());
        for (i, &val) in data.iter().enumerate() {
            v.set(i, val);
        }
        Self { inner: v }
    }

    /// Get the length.
    #[wasm_bindgen(getter)]
    pub fn len(&self) -> usize {
        self.inner.rows()
    }

    /// Check if empty.
    #[wasm_bindgen(js_name = isEmpty)]
    pub fn is_empty(&self) -> bool {
        self.inner.rows() == 0
    }

    /// Get element at index.
    pub fn get(&self, i: usize) -> f64 {
        self.inner.get(i)
    }

    /// Set element at index.
    pub fn set(&mut self, i: usize, value: f64) {
        self.inner.set(i, value);
    }

    /// Return data as Float64Array.
    #[wasm_bindgen(js_name = toArray)]
    pub fn to_array(&self) -> Vec<f64> {
        self.inner.data().to_vec()
    }

    /// Dot product with another vector.
    pub fn dot(&self, other: &WasmVector) -> Result<f64, JsError> {
        if self.len() != other.len() {
            return Err(JsError::new("Vector lengths must match for dot product"));
        }
        Ok(self.inner.dot(&other.inner))
    }

    /// Euclidean norm.
    pub fn norm(&self) -> f64 {
        self.inner.norm()
    }

    /// Vector addition.
    pub fn add(&self, other: &WasmVector) -> Result<WasmVector, JsError> {
        if self.len() != other.len() {
            return Err(JsError::new("Vector lengths must match for addition"));
        }
        Ok(Self {
            inner: &self.inner + &other.inner,
        })
    }

    /// Vector subtraction.
    pub fn sub(&self, other: &WasmVector) -> Result<WasmVector, JsError> {
        if self.len() != other.len() {
            return Err(JsError::new("Vector lengths must match for subtraction"));
        }
        Ok(Self {
            inner: &self.inner - &other.inner,
        })
    }

    /// Scalar multiplication.
    pub fn scale(&self, scalar: f64) -> WasmVector {
        let mut result = Vector::with_capacity(self.len());
        for i in 0..self.len() {
            result.set(i, self.inner.get(i) * scalar);
        }
        Self { inner: result }
    }

    /// Linear interpolation with another vector: (1 - t) * self + t * other.
    pub fn lerp(&self, other: &WasmVector, t: f64) -> Result<WasmVector, JsError> {
        if self.len() != other.len() {
            return Err(JsError::new("Vector lengths must match for lerp"));
        }
        let mut out = Vector::with_capacity(self.len());
        for i in 0..self.len() {
            let a = self.inner.get(i);
            let b = other.inner.get(i);
            out.set(i, crate::easing::lerp(a, b, t));
        }
        Ok(Self { inner: out })
    }

    /// Euclidean distance to another vector.
    #[wasm_bindgen(js_name = euclideanDistance)]
    pub fn euclidean_distance(&self, other: &WasmVector) -> Result<f64, JsError> {
        if self.len() != other.len() {
            return Err(JsError::new("Vector lengths must match for distance"));
        }
        Ok(euclidean(&self.inner, &other.inner))
    }
}

// ============================================================================
// WasmSvd — SVD result (U, V, sigma) for JavaScript
// ============================================================================

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

// ============================================================================
// WasmMatrix32 — f32 matrix for 3D graphics (matches math3d types)
// ============================================================================

/// A 32-bit float matrix for 3D graphics operations.
#[wasm_bindgen]
pub struct WasmMatrix32 {
    inner: Matrix<f32>,
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
        // make_rotation returns Matrix3f (3x3), we embed in 4x4
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
}

// ============================================================================
// WasmCg — Camera and projection helpers for JavaScript
// ============================================================================

/// Camera and projection matrix builders (4×4, column-major).
#[wasm_bindgen(js_name = WasmCg)]
pub struct WasmCg;

#[wasm_bindgen]
impl WasmCg {
    /// Right-handed look-at view matrix: eye (x,y,z), target (x,y,z), up (x,y,z).
    #[wasm_bindgen(js_name = lookAtRh)]
    pub fn look_at_rh(
        eye_x: f32,
        eye_y: f32,
        eye_z: f32,
        target_x: f32,
        target_y: f32,
        target_z: f32,
        up_x: f32,
        up_y: f32,
        up_z: f32,
    ) -> WasmMatrix32 {
        let eye = crate::vector3(eye_x, eye_y, eye_z);
        let target = crate::vector3(target_x, target_y, target_z);
        let up = crate::vector3(up_x, up_y, up_z);
        WasmMatrix32 {
            inner: look_at_rh(&eye, &target, &up),
        }
    }

    /// Left-handed look-at view matrix.
    #[wasm_bindgen(js_name = lookAtLh)]
    pub fn look_at_lh(
        eye_x: f32,
        eye_y: f32,
        eye_z: f32,
        target_x: f32,
        target_y: f32,
        target_z: f32,
        up_x: f32,
        up_y: f32,
        up_z: f32,
    ) -> WasmMatrix32 {
        let eye = crate::vector3(eye_x, eye_y, eye_z);
        let target = crate::vector3(target_x, target_y, target_z);
        let up = crate::vector3(up_x, up_y, up_z);
        WasmMatrix32 {
            inner: look_at_lh(&eye, &target, &up),
        }
    }

    /// Perspective projection: aspect = width/height, fov_y_rad vertical FOV (radians), near/far positive.
    #[wasm_bindgen(js_name = newPerspective)]
    pub fn new_perspective(aspect: f32, fov_y_rad: f32, near: f32, far: f32) -> WasmMatrix32 {
        WasmMatrix32 {
            inner: new_perspective(aspect, fov_y_rad, near, far),
        }
    }

    /// Orthographic projection: left, right, bottom, top, near, far.
    #[wasm_bindgen(js_name = newOrthographic)]
    pub fn new_orthographic(
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    ) -> WasmMatrix32 {
        WasmMatrix32 {
            inner: new_orthographic(left, right, bottom, top, near, far),
        }
    }

    /// Translation matrix with (x, y, z).
    #[wasm_bindgen(js_name = newTranslation)]
    pub fn new_translation(x: f32, y: f32, z: f32) -> WasmMatrix32 {
        let t = crate::vector3(x, y, z);
        WasmMatrix32 {
            inner: new_translation(&t),
        }
    }
}
