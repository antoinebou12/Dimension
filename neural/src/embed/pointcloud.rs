//! Point cloud embedding via a user-supplied ONNX model (e.g. PointNet).
//!
//! Load an ONNX model that takes point cloud input (N×C, e.g. N×3 xyz) and returns a single embedding vector.

#[cfg(feature = "onnx")]
use ort::Session;
use std::path::Path;

/// Point cloud embedding session: runs ONNX on point cloud (N×C) and returns one embedding vector.
#[cfg(feature = "onnx")]
pub struct PointCloudEmbedding {
    session: Session,
    input_size: usize,
    output_size: usize,
}

#[cfg(feature = "onnx")]
impl PointCloudEmbedding {
    /// Load ONNX model from path. Model input: shape [1, N, C] or [1, N*C]; output: [1, D].
    pub fn from_onnx_path(path: &Path) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let session = Session::builder()
            .commit_from_file(path)
            .map_err(|e| format!("onnx load: {}", e))?;
        let input = session.inputs.first().ok_or("no input")?;
        let output = session.outputs.first().ok_or("no output")?;
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
        Ok(Self {
            session,
            input_size,
            output_size,
        })
    }

    pub fn from_onnx_path_str(
        path: &str,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Self::from_onnx_path(Path::new(path))
    }

    #[must_use]
    pub fn input_size(&self) -> usize {
        self.input_size
    }

    #[must_use]
    pub fn output_dimension(&self) -> usize {
        self.output_size
    }

    /// Run inference. `points` must have length `input_size()` (e.g. N*3 for xyz).
    pub fn embed(
        &self,
        points: &[f32],
    ) -> Result<Vec<f32>, Box<dyn std::error::Error + Send + Sync>> {
        if points.len() != self.input_size {
            return Err(
                format!("expected {} floats, got {}", self.input_size, points.len()).into(),
            );
        }
        let shape: Vec<i64> = self
            .session
            .inputs
            .first()
            .map(|i| i.dimensions.clone())
            .unwrap_or_else(|| vec![1, self.input_size as i64]);
        let input_tensor =
            ort::Value::from_array(self.session.allocator(), (shape.as_slice(), points))
                .map_err(|e| format!("tensor: {}", e))?;
        let outputs = self
            .session
            .run(vec![input_tensor])
            .map_err(|e| format!("run: {}", e))?;
        let out = outputs
            .first()
            .ok_or("no output")?
            .extract_tensor::<f32>()
            .map_err(|e| format!("extract: {}", e))?;
        let slice = out.view().as_slice().map_err(|e| format!("view: {}", e))?;
        Ok(slice.to_vec())
    }
}

#[cfg(not(feature = "onnx"))]
pub struct PointCloudEmbedding;

#[cfg(not(feature = "onnx"))]
impl PointCloudEmbedding {
    pub fn from_onnx_path(_path: &Path) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Err("pointcloud embedding requires feature onnx".into())
    }
    pub fn from_onnx_path_str(
        _path: &str,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Err("pointcloud embedding requires feature onnx".into())
    }
    pub fn input_size(&self) -> usize {
        0
    }
    pub fn output_dimension(&self) -> usize {
        0
    }
    pub fn embed(
        &self,
        _points: &[f32],
    ) -> Result<Vec<f32>, Box<dyn std::error::Error + Send + Sync>> {
        Err("pointcloud embedding requires feature onnx".into())
    }
}
