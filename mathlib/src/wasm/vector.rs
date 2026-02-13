//! WasmVector — Vector wrapper for JavaScript.

use wasm_bindgen::prelude::*;

use crate::Vector;
use crate::distance::euclidean;

/// A vector accessible from JavaScript.
#[wasm_bindgen]
pub struct WasmVector {
    pub(crate) inner: Vector<f64>,
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

/// Dot product of two f32 vectors (uses GPU when available and above threshold).
#[wasm_bindgen(js_name = dotF32)]
pub fn dot_f32(a: &[f32], b: &[f32]) -> Result<f32, JsError> {
    if a.len() != b.len() {
        return Err(JsError::new("Vector lengths must match for dot product"));
    }
    let va = Vector::<f32>::from_slice(a);
    let vb = Vector::<f32>::from_slice(b);
    Ok(va.dot(&vb))
}
