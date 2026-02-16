//! ONNX model loading and inference for neural IK.
//!
//! With the `onnx` feature, use the `ort` crate to load an ONNX file and run inference.
//! Input: target position (3 floats) or position + current state (3 + dof). Output: joint angles (dof).

#[cfg(feature = "onnx")]
use ort::Session;

#[cfg(feature = "onnx")]
/// Session for running a neural IK ONNX model (load with ONNX Runtime via `ort`).
pub struct OnnxIkSession {
    session: Session,
    input_size: usize,
    output_size: usize,
}

#[cfg(feature = "onnx")]
impl OnnxIkSession {
    /// Load an ONNX model from path. Returns None on error.
    pub fn load(path: &std::path::Path) -> Option<Self> {
        let session = Session::builder().commit_from_file(path).ok()?;
        let input = session.inputs.first()?;
        let output = session.outputs.first()?;
        let input_size: usize = input
            .dimensions
            .iter()
            .map(|&d| d.max(0) as usize)
            .product();
        let output_size: usize = output
            .dimensions
            .iter()
            .map(|&d| d.max(0) as usize)
            .product();
        Some(Self {
            session,
            input_size,
            output_size,
        })
    }

    /// Load from path string.
    pub fn load_path(path: &str) -> Option<Self> {
        Self::load(std::path::Path::new(path))
    }

    /// Input dimension expected by the model.
    #[must_use]
    pub fn input_size(&self) -> usize {
        self.input_size
    }

    /// Output dimension (dof).
    #[must_use]
    pub fn output_size(&self) -> usize {
        self.output_size
    }

    /// Run inference. Input length must match `input_size()`. Returns empty vec on error.
    #[must_use]
    pub fn predict(&self, input: &[f32]) -> Vec<f32> {
        if input.len() != self.input_size {
            return vec![];
        }
        let input_name = self
            .session
            .inputs
            .first()
            .and_then(|i| i.name.as_deref())
            .unwrap_or("input");
        let output_name = self
            .session
            .outputs
            .first()
            .and_then(|o| o.name.as_deref())
            .unwrap_or("output");
        let shape: Vec<i64> = [1, self.input_size as i64].to_vec();
        let input_tensor =
            ort::Value::from_array(self.session.allocator(), (shape.as_slice(), input)).ok()?;
        let outputs = self.session.run(vec![input_tensor]).ok()?;
        let out = outputs.first()?.extract_tensor::<f32>().ok()?;
        out.view()
            .as_slice()
            .ok()
            .map(|s| s.to_vec())
            .unwrap_or_default()
    }
}

/// Placeholder when `onnx` feature is disabled.
#[cfg(not(feature = "onnx"))]
pub struct OnnxIkSession;

#[cfg(not(feature = "onnx"))]
impl OnnxIkSession {
    /// Build with `onnx` feature to load and run ONNX models.
    #[must_use]
    pub fn load_path(_path: &str) -> Option<Self> {
        None
    }

    /// Run inference (no-op when onnx disabled).
    #[must_use]
    pub fn predict(&self, _input: &[f32]) -> Vec<f32> {
        vec![]
    }
}
