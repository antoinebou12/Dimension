//! WasmSimplexResult — Simplex LP solver for JavaScript.
//!
//! Solves LPs in standard form: minimize c'x subject to Ax = b, x >= 0.

use wasm_bindgen::prelude::*;

use crate::simplex::{SimplexStatus, simplex_solve};

use super::matrix::WasmMatrix;
use super::vector::WasmVector;

/// Result of simplex LP solve: solution vector, objective value, and status string.
#[wasm_bindgen(js_name = WasmSimplexResult)]
pub struct WasmSimplexResult {
    x: WasmVector,
    objective: f64,
    status: String,
}

#[wasm_bindgen]
impl WasmSimplexResult {
    /// Solve LP in standard form: minimize c'x subject to Ax = b, x >= 0.
    /// Takes objective coefficients `c`, constraint matrix `A`, and RHS `b`.
    /// Returns solution vector, objective value, and status ("optimal", "unbounded", "infeasible", or error message).
    #[wasm_bindgen(constructor)]
    pub fn new(
        c: &WasmVector,
        a: &WasmMatrix,
        b: &WasmVector,
    ) -> Result<WasmSimplexResult, JsError> {
        let result = simplex_solve(&c.inner, &a.inner, &b.inner)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let status = match result.status {
            SimplexStatus::Optimal => "optimal".to_string(),
            SimplexStatus::Unbounded => "unbounded".to_string(),
            SimplexStatus::Infeasible => "infeasible".to_string(),
        };
        Ok(WasmSimplexResult {
            x: WasmVector { inner: result.x },
            objective: result.objective,
            status,
        })
    }

    /// Solution vector x (length n).
    #[wasm_bindgen(js_name = getX)]
    pub fn get_x(&self) -> WasmVector {
        WasmVector {
            inner: self.x.inner.clone(),
        }
    }

    /// Optimal objective value (c'x).
    #[wasm_bindgen(js_name = getObjective)]
    pub fn get_objective(&self) -> f64 {
        self.objective
    }

    /// Status string: "optimal", "unbounded", or "infeasible".
    #[wasm_bindgen(js_name = getStatus)]
    pub fn get_status(&self) -> String {
        self.status.clone()
    }
}
