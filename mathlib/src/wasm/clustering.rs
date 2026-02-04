//! `WasmKmeans` and `WasmDbscan` — clustering for JavaScript.

use wasm_bindgen::prelude::*;

use crate::clustering::{NOISE, dbscan, kmeans};

use super::matrix::WasmMatrix;

/// Result of K-means clustering: labels and centroids.
#[wasm_bindgen]
pub struct WasmKmeans {
    labels: Vec<usize>,
    centroids: crate::Matrix<f64>,
}

#[wasm_bindgen]
impl WasmKmeans {
    /// Run K-means on data matrix (rows = samples, cols = features).
    /// `k` is the number of clusters. `max_iters` is maximum iterations (0 = 300).
    #[wasm_bindgen(constructor)]
    pub fn new(data: &WasmMatrix, k: usize, max_iters: u32) -> Result<WasmKmeans, JsError> {
        if k == 0 {
            return Err(JsError::new("k must be at least 1"));
        }
        if data.rows() < k {
            return Err(JsError::new(&format!(
                "Data has {} samples but k={}",
                data.rows(),
                k
            )));
        }
        let iters = if max_iters == 0 {
            None
        } else {
            Some(max_iters)
        };
        let result = kmeans(&data.inner, k, iters);
        Ok(Self {
            labels: result.labels().to_vec(),
            centroids: result.centroids().clone(),
        })
    }

    /// Cluster label for each sample (0 to k-1).
    #[wasm_bindgen(js_name = getLabels)]
    pub fn get_labels(&self) -> Vec<usize> {
        self.labels.clone()
    }

    /// Centroid matrix (k rows × features columns).
    #[wasm_bindgen(js_name = getCentroids)]
    pub fn get_centroids(&self) -> WasmMatrix {
        WasmMatrix {
            inner: self.centroids.clone(),
        }
    }

    /// Number of clusters.
    #[wasm_bindgen(js_name = nClusters)]
    pub fn n_clusters(&self) -> usize {
        self.centroids.rows()
    }
}

/// Label for DBSCAN noise points (not assigned to any cluster).
/// In JS this is the maximum 32-bit unsigned value when cast from u32.
#[wasm_bindgen(js_name = NOISE_LABEL)]
pub fn dbscan_noise_label() -> u32 {
    // NOISE is usize::MAX; for JS we expose as u32::MAX for typical use.
    u32::MAX
}

/// Result of DBSCAN clustering: labels (cluster index or noise).
#[wasm_bindgen]
pub struct WasmDbscan {
    labels: Vec<usize>,
}

#[wasm_bindgen]
impl WasmDbscan {
    /// Run DBSCAN on data matrix (rows = samples, cols = features).
    /// Points within `eps` (Euclidean) are neighbors; core points have at least `min_pts` neighbors.
    #[wasm_bindgen(constructor)]
    pub fn new(data: &WasmMatrix, eps: f64, min_pts: usize) -> Result<WasmDbscan, JsError> {
        if eps <= 0.0 {
            return Err(JsError::new("eps must be positive"));
        }
        if min_pts < 1 {
            return Err(JsError::new("min_pts must be at least 1"));
        }
        let result = dbscan(&data.inner, eps, min_pts);
        Ok(Self {
            labels: result.labels().to_vec(),
        })
    }

    /// Cluster label for each sample (0, 1, ...) or `NOISE_LABEL` for noise.
    #[wasm_bindgen(js_name = getLabels)]
    pub fn get_labels(&self) -> Vec<usize> {
        self.labels.clone()
    }

    /// Number of clusters (excluding noise).
    #[wasm_bindgen(js_name = nClusters)]
    pub fn n_clusters(&self) -> usize {
        self.labels
            .iter()
            .filter(|&&l| l != NOISE)
            .max()
            .map_or(0, |&m| m + 1)
    }

    /// Whether sample `i` is classified as noise.
    #[wasm_bindgen(js_name = isNoise)]
    pub fn is_noise(&self, i: usize) -> bool {
        self.labels.get(i).is_some_and(|&l| l == NOISE)
    }
}
