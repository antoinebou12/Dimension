//! Text embedding via fastembed (e.g. MiniLM).

use fastembed::{EmbeddingModel, InitOptions, TextEmbedding as FastembedText};

/// Text embedding model (fastembed backend).
pub struct TextEmbedding {
    model: FastembedText,
}

impl TextEmbedding {
    /// Create with default options (e.g. MiniLM).
    pub fn try_new_default() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let model = FastembedText::try_new(Default::default())?;
        Ok(Self { model })
    }

    /// Create with MiniLM L6 V2 (384 dims), good for demos and recommendation.
    pub fn try_new_minilm() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let opts =
            InitOptions::new(EmbeddingModel::AllMiniLML6V2).with_show_download_progress(false);
        let model = FastembedText::try_new(opts)?;
        Ok(Self { model })
    }

    /// Create with custom init options.
    pub fn try_new(opts: InitOptions) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let model = FastembedText::try_new(opts)?;
        Ok(Self { model })
    }

    /// Embed a list of texts. Returns one vector per input.
    pub fn embed(
        &self,
        texts: &[&str],
        batch_size: Option<usize>,
    ) -> Result<Vec<Vec<f32>>, Box<dyn std::error::Error + Send + Sync>> {
        let documents: Vec<String> = texts.iter().map(|s| (*s).to_string()).collect();
        let refs: Vec<&str> = documents.iter().map(String::as_str).collect();
        let embeddings = self.model.embed(refs, batch_size)?;
        Ok(embeddings)
    }

    /// Embedding dimension (e.g. 384 for MiniLM L6).
    pub fn dimension(&self) -> usize {
        self.model.embedding_dim()
    }
}
