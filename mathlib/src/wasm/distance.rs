//! WasmDistance — Distance functions for JavaScript.

use wasm_bindgen::prelude::*;

use crate::{chebyshev, cosine_distance, cosine_similarity, manhattan, minkowski};

use super::vector::WasmVector;

/// Distance metric functions.
#[wasm_bindgen(js_name = WasmDistance)]
pub struct WasmDistance;

#[wasm_bindgen]
impl WasmDistance {
    /// Manhattan (L1) distance between two vectors.
    #[wasm_bindgen(js_name = manhattan)]
    pub fn manhattan(a: &WasmVector, b: &WasmVector) -> Result<f64, JsError> {
        if a.len() != b.len() {
            return Err(JsError::new("Vector lengths must match"));
        }
        Ok(manhattan(&a.inner, &b.inner))
    }

    /// Cosine similarity between two vectors (1 = identical direction, 0 = orthogonal, -1 = opposite).
    #[wasm_bindgen(js_name = cosineSimilarity)]
    pub fn cosine_similarity(a: &WasmVector, b: &WasmVector) -> Result<f64, JsError> {
        if a.len() != b.len() {
            return Err(JsError::new("Vector lengths must match"));
        }
        Ok(cosine_similarity(&a.inner, &b.inner))
    }

    /// Cosine distance between two vectors (1 - cosine_similarity).
    #[wasm_bindgen(js_name = cosineDistance)]
    pub fn cosine_distance(a: &WasmVector, b: &WasmVector) -> Result<f64, JsError> {
        if a.len() != b.len() {
            return Err(JsError::new("Vector lengths must match"));
        }
        Ok(cosine_distance(&a.inner, &b.inner))
    }

    /// Chebyshev (L-infinity) distance between two vectors.
    #[wasm_bindgen(js_name = chebyshev)]
    pub fn chebyshev(a: &WasmVector, b: &WasmVector) -> Result<f64, JsError> {
        if a.len() != b.len() {
            return Err(JsError::new("Vector lengths must match"));
        }
        Ok(chebyshev(&a.inner, &b.inner))
    }

    /// Minkowski distance with exponent p.
    #[wasm_bindgen(js_name = minkowski)]
    pub fn minkowski(a: &WasmVector, b: &WasmVector, p: f64) -> Result<f64, JsError> {
        if a.len() != b.len() {
            return Err(JsError::new("Vector lengths must match"));
        }
        Ok(minkowski(&a.inner, &b.inner, p))
    }
}
