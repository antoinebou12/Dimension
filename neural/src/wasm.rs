//! WASM bindings for neural IK inference.
//!
//! Use with `wasm-pack build --target web --features wasm` for browser.
//! Inference can use Burn (ndarray backend) or a pre-loaded ONNX model.

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

/// WASM-facing neural IK wrapper for inference.
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct NeuralIkWasm {
    /// Normalized target position (3) or position + current state (3 + dof).
    input_buf: Vec<f32>,
    /// Chain DOF (output size).
    dof: usize,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl NeuralIkWasm {
    /// Create a placeholder for inference (model must be set via set_weights or ONNX).
    #[wasm_bindgen(constructor)]
    pub fn new(dof: usize, use_current_state: bool) -> Self {
        let input_len = if use_current_state { 3 + dof } else { 3 };
        Self {
            input_buf: vec![0.0; input_len],
            dof,
        }
    }

    /// Set target position (x, y, z) for next prediction.
    #[wasm_bindgen(js_name = setTarget)]
    pub fn set_target(&mut self, x: f32, y: f32, z: f32) {
        if self.input_buf.len() >= 3 {
            self.input_buf[0] = x;
            self.input_buf[1] = y;
            self.input_buf[2] = z;
        }
    }

    /// Set current joint state (length must match dof when use_current_state was true).
    #[wasm_bindgen(js_name = setCurrentJoints)]
    pub fn set_current_joints(&mut self, joints: &[f32]) {
        let start = 3;
        let end = (start + self.dof)
            .min(self.input_buf.len())
            .min(start + joints.len());
        for (i, &v) in joints.iter().take(end - start).enumerate() {
            self.input_buf[start + i] = v;
        }
    }

    /// Return the current input buffer (for use with external model inference).
    #[wasm_bindgen(js_name = getInput)]
    pub fn get_input(&self) -> Vec<f32> {
        self.input_buf.clone()
    }

    /// DOF (number of output joints).
    #[wasm_bindgen(js_name = dof)]
    pub fn dof(&self) -> usize {
        self.dof
    }
}
