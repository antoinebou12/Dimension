//! Image embedding via fastembed (e.g. CLIP).

use std::path::Path;

use fastembed::{ImageEmbedding as FastembedImage, ImageEmbeddingModel, ImageInitOptions};

/// Image embedding model (fastembed backend).
pub struct ImageEmbedding {
    model: FastembedImage,
}

impl ImageEmbedding {
    /// Create with default options (e.g. CLIP ViT-B/32).
    pub fn try_new_default() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let model = FastembedImage::try_new(Default::default())?;
        Ok(Self { model })
    }

    /// Create with CLIP ViT-B/32 (512 dims).
    pub fn try_new_clip() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let opts = ImageInitOptions::new(ImageEmbeddingModel::ClipVitB32)
            .with_show_download_progress(false);
        let model = FastembedImage::try_new(opts)?;
        Ok(Self { model })
    }

    /// Create with custom init options.
    pub fn try_new(
        opts: ImageInitOptions,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let model = FastembedImage::try_new(opts)?;
        Ok(Self { model })
    }

    /// Embed images from file paths.
    pub fn embed_paths<P: AsRef<Path>>(
        &self,
        paths: &[P],
        batch_size: Option<usize>,
    ) -> Result<Vec<Vec<f32>>, Box<dyn std::error::Error + Send + Sync>> {
        let paths_str: Vec<String> = paths
            .iter()
            .map(|p| p.as_ref().display().to_string())
            .collect();
        let refs: Vec<&str> = paths_str.iter().map(String::as_str).collect();
        let embeddings = self.model.embed(refs, batch_size)?;
        Ok(embeddings)
    }

    /// Embed images from raw bytes (e.g. PNG/JPEG). Writes to temp files and calls embed_paths; prefer paths when possible.
    pub fn embed_bytes(
        &self,
        images: &[&[u8]],
        batch_size: Option<usize>,
    ) -> Result<Vec<Vec<f32>>, Box<dyn std::error::Error + Send + Sync>> {
        let temp_dir = std::env::temp_dir().join("neural_embed_images");
        std::fs::create_dir_all(&temp_dir)?;
        let mut paths = Vec::with_capacity(images.len());
        for (i, bytes) in images.iter().enumerate() {
            let path = temp_dir.join(format!("img_{}.bin", i));
            std::fs::write(&path, bytes)?;
            paths.push(path);
        }
        let path_refs: Vec<&std::path::Path> = paths.iter().map(|p| p.as_path()).collect();
        let result = self.embed_paths(&path_refs, batch_size);
        for p in &paths {
            let _ = std::fs::remove_file(p);
        }
        result
    }

    /// Embedding dimension (e.g. 512 for CLIP ViT-B/32).
    pub fn dimension(&self) -> usize {
        self.model.embedding_dim()
    }
}
