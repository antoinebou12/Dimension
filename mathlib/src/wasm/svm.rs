//! WasmSvm, WasmSvmResult, WasmSvmRbf, WasmSvmRbfResult — Linear and RBF SVM for JavaScript.

use wasm_bindgen::prelude::*;

use crate::svm;

use super::matrix::WasmMatrix;
use super::vector::WasmVector;

/// Trained linear SVM: weight vector and bias for prediction sign(w·x + b).
#[wasm_bindgen]
pub struct WasmSvmResult {
    inner: crate::SvmResult,
}

#[wasm_bindgen]
impl WasmSvmResult {
    /// Weight vector (one per feature).
    #[wasm_bindgen(js_name = getWeights)]
    pub fn get_weights(&self) -> WasmVector {
        WasmVector {
            inner: self.inner.weights().clone(),
        }
    }

    /// Bias term.
    #[wasm_bindgen(js_name = getBias)]
    pub fn get_bias(&self) -> f64 {
        self.inner.bias()
    }

    /// Predict label for one sample: +1 or -1. `sample` is a row of features (length = n_features).
    #[wasm_bindgen(js_name = predict)]
    pub fn predict(&self, sample: &[f64]) -> f64 {
        self.inner.predict_row(sample)
    }

    /// Predict labels for all rows of X. Returns array of +1 or -1.
    #[wasm_bindgen(js_name = predictAll)]
    pub fn predict_all(&self, x: &WasmMatrix) -> Vec<f64> {
        self.inner.predict(&x.inner)
    }
}

/// Linear SVM (binary classification). Train with X (rows = samples, cols = features) and labels ±1.
#[wasm_bindgen(js_name = WasmSvm)]
pub struct WasmSvm;

#[wasm_bindgen]
impl WasmSvm {
    /// Train linear SVM. Labels must be ±1 (or positive → 1, else -1). Uses default options.
    #[wasm_bindgen(js_name = train)]
    pub fn train(x: &WasmMatrix, labels: &[f64]) -> Result<WasmSvmResult, JsError> {
        let result = svm(&x.inner, labels, None).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmSvmResult { inner: result })
    }
}

/// Trained RBF-kernel SVM: support vectors, dual coefficients, bias, γ. Prediction via sign(Σ αᵢyᵢ K(svᵢ,x) + b).
#[wasm_bindgen(js_name = WasmSvmRbfResult)]
pub struct WasmSvmRbfResult {
    inner: crate::SvmRbfResult,
}

#[wasm_bindgen]
impl WasmSvmRbfResult {
    /// Support vectors matrix (n_sv × n_features).
    #[wasm_bindgen(js_name = getSupportVectors)]
    pub fn get_support_vectors(&self) -> WasmMatrix {
        WasmMatrix {
            inner: self.inner.support_vectors().clone(),
        }
    }

    /// Bias term.
    #[wasm_bindgen(js_name = getBias)]
    pub fn get_bias(&self) -> f64 {
        self.inner.bias()
    }

    /// RBF kernel parameter γ.
    #[wasm_bindgen(js_name = getGamma)]
    pub fn get_gamma(&self) -> f64 {
        self.inner.gamma()
    }

    /// Predict label for one sample: +1 or -1.
    #[wasm_bindgen(js_name = predict)]
    pub fn predict(&self, sample: &[f64]) -> f64 {
        self.inner.predict_row(sample)
    }

    /// Predict labels for all rows of X. Returns array of +1 or -1.
    #[wasm_bindgen(js_name = predictAll)]
    pub fn predict_all(&self, x: &WasmMatrix) -> Vec<f64> {
        self.inner.predict(&x.inner)
    }
}

/// RBF-kernel SVM (binary classification). Train with X, labels ±1, and gamma.
#[wasm_bindgen(js_name = WasmSvmRbf)]
pub struct WasmSvmRbf;

#[wasm_bindgen]
impl WasmSvmRbf {
    /// Train RBF SVM. Labels ±1. Gamma controls kernel width (e.g. 0.5).
    #[wasm_bindgen(js_name = train)]
    pub fn train(x: &WasmMatrix, labels: &[f64], gamma: f64) -> Result<WasmSvmRbfResult, JsError> {
        let result = crate::svm::svm_rbf(&x.inner, labels, gamma, None)
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmSvmRbfResult { inner: result })
    }
}
