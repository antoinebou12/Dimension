//! Precompute text embeddings for the WASM recommendation demo (MiniLM).
//!
//! Run from repo root:
//!   cargo run -p neural --example precompute_demo_embeddings --features precompute -- mathlib/demo/wasm-demo/recommendation/data.json
//! Or to print JSON to stdout:
//!   cargo run -p neural --example precompute_demo_embeddings --features precompute

use neural::TextEmbedding;
use std::env;
use std::fs;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let model = TextEmbedding::try_new_minilm()?;
    let texts: Vec<&str> = vec![
        "Rust is a systems programming language.",
        "Machine learning models can be exported to ONNX.",
        "WebAssembly runs in the browser.",
        "Vector embeddings are used for search and recommendation.",
        "PCA reduces high-dimensional data to fewer dimensions.",
        "The neural crate provides embedding and ONNX inference.",
        "Fastembed supports text and image embeddings.",
        "Recommendation systems use similarity between vectors.",
        "Dimensionality reduction helps visualize high-D data.",
        "Python is popular for data science.",
        "Rust offers memory safety without garbage collection.",
        "Graph neural networks operate on graph structures.",
        "Point clouds represent 3D geometry.",
        "PDF documents can be parsed to extract text.",
        "Cosine similarity measures angle between vectors.",
    ];
    let embeddings = model.embed(&texts, None)?;
    let json = serde_json::json!({
        "texts": texts,
        "embeddings": embeddings,
    });
    let json_str = serde_json::to_string_pretty(&json)?;
    let args: Vec<String> = env::args().collect();
    if args.len() >= 2 {
        let path = &args[1];
        fs::write(path, json_str)?;
        eprintln!("Wrote {} texts and embeddings to {}", texts.len(), path);
    } else {
        let mut out = std::io::stdout().lock();
        out.write_all(json_str.as_bytes())?;
        out.flush()?;
    }
    Ok(())
}
