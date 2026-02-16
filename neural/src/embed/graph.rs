//! Graph embedding via ONNX (e.g. GNN). Model is typically downloaded by the `download_models` binary.
//!
//! Expects ONNX input: node features (and optionally edge index). Output: node embeddings (one vector per node).

#[cfg(feature = "onnx")]
use ort::Session;
use std::path::Path;

/// Graph embedding session: runs ONNX on graph (node features, optional edges) and returns node embeddings.
#[cfg(feature = "onnx")]
pub struct GraphEmbedding {
    session: Session,
    /// Number of nodes × feature dim for primary input (node features).
    node_input_size: usize,
    /// Output size per node (embedding dimension).
    output_dim: usize,
}

#[cfg(feature = "onnx")]
impl GraphEmbedding {
    /// Load ONNX from path. Typical input: node features [N, F]; optional edge_index [2, E]. Output: [N, D].
    pub fn from_onnx_path(path: &Path) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let session = Session::builder()
            .commit_from_file(path)
            .map_err(|e| format!("onnx load: {}", e))?;
        let input = session.inputs.first().ok_or("no input")?;
        let output = session.outputs.first().ok_or("no output")?;
        let node_input_size: usize = input
            .dimensions
            .iter()
            .map(|&d| d.max(0) as usize)
            .product();
        let output_size: usize = output
            .dimensions
            .iter()
            .map(|&d| d.max(0) as usize)
            .product();
        // If output is [N, D], output_size = N*D; we treat output_dim as D (single node embedding dim).
        let output_dim = if output.dimensions.len() >= 2 {
            output
                .dimensions
                .last()
                .map(|&d| d.max(0) as usize)
                .unwrap_or(output_size)
        } else {
            output_size
        };
        Ok(Self {
            session,
            node_input_size,
            output_dim,
        })
    }

    pub fn from_onnx_path_str(
        path: &str,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Self::from_onnx_path(Path::new(path))
    }

    /// Expected number of node feature floats (N*F).
    #[must_use]
    pub fn node_input_size(&self) -> usize {
        self.node_input_size
    }

    #[must_use]
    pub fn output_dimension(&self) -> usize {
        self.output_dim
    }

    /// Run inference with node features only. `node_features` length must match model input (e.g. N*F).
    /// Returns flattened node embeddings (row-major N×D).
    pub fn embed_nodes(
        &self,
        node_features: &[f32],
    ) -> Result<Vec<f32>, Box<dyn std::error::Error + Send + Sync>> {
        if node_features.len() != self.node_input_size {
            return Err(format!(
                "expected {} node feature floats, got {}",
                self.node_input_size,
                node_features.len()
            )
            .into());
        }
        let shape: Vec<i64> = self
            .session
            .inputs
            .first()
            .map(|i| i.dimensions.clone())
            .unwrap_or_else(|| vec![1, self.node_input_size as i64]);
        let input_tensor =
            ort::Value::from_array(self.session.allocator(), (shape.as_slice(), node_features))
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

    /// Return embeddings as rows (one Vec<f32> per node). Assumes output shape [N, D].
    pub fn embed_nodes_rows(
        &self,
        node_features: &[f32],
    ) -> Result<Vec<Vec<f32>>, Box<dyn std::error::Error + Send + Sync>> {
        let flat = self.embed_nodes(node_features)?;
        let n = flat.len() / self.output_dim;
        let mut rows = Vec::with_capacity(n);
        for i in 0..n {
            rows.push(flat[i * self.output_dim..(i + 1) * self.output_dim].to_vec());
        }
        Ok(rows)
    }
}

#[cfg(not(feature = "onnx"))]
pub struct GraphEmbedding;

#[cfg(not(feature = "onnx"))]
impl GraphEmbedding {
    pub fn from_onnx_path(_path: &Path) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Err("graph embedding requires feature onnx".into())
    }

    pub fn from_onnx_path_str(
        _path: &str,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Err("graph embedding requires feature onnx".into())
    }

    pub fn node_input_size(&self) -> usize {
        0
    }

    pub fn output_dimension(&self) -> usize {
        0
    }

    pub fn embed_nodes(
        &self,
        _node_features: &[f32],
    ) -> Result<Vec<f32>, Box<dyn std::error::Error + Send + Sync>> {
        Err("graph embedding requires feature onnx".into())
    }

    pub fn embed_nodes_rows(
        &self,
        _node_features: &[f32],
    ) -> Result<Vec<Vec<f32>>, Box<dyn std::error::Error + Send + Sync>> {
        Err("graph embedding requires feature onnx".into())
    }
}
