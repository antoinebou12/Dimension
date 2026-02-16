//! Download ONNX models into neural/models/ for use by the neural crate.
//!
//! Usage:
//!   cargo run --bin download_models -- [--graph] [--all]
//!   From repo root: cargo run -p neural --bin download_models -- --all

#[cfg(target_arch = "wasm32")]
fn main() {}

#[cfg(not(target_arch = "wasm32"))]
mod real {
    use clap::{CommandFactory, Parser};
    use std::io::Write;
    use std::path::Path;

    const HF_GRAPH_REPO: &str = "vishnun/quantized_knowledge_graph_nlp_onnx";
    const HF_GRAPH_FILE: &str = "model_quantized.onnx";

    pub fn run() -> Result<(), Box<dyn std::error::Error>> {
        let cli = Cli::parse();

        if !cli.graph && !cli.all {
            Cli::command().print_help()?;
            return Ok(());
        }

        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let models_dir = manifest_dir.join("models");
        std::fs::create_dir_all(&models_dir)?;

        if cli.graph || cli.all {
            download_graph_onnx(&models_dir)?;
        }

        println!("Models dir: {}", models_dir.display());
        Ok(())
    }

    #[derive(Parser)]
    #[command(name = "download_models")]
    #[command(about = "Download ONNX models into neural/models/")]
    struct Cli {
        #[arg(long)]
        graph: bool,
        #[arg(long)]
        all: bool,
    }

    fn download_graph_onnx(models_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!(
            "https://huggingface.co/{}/resolve/main/{}",
            HF_GRAPH_REPO, HF_GRAPH_FILE
        );
        let out_path = models_dir.join(HF_GRAPH_FILE);

        println!("Downloading {} ...", url);
        let client = reqwest::blocking::Client::builder()
            .redirect(reqwest::redirect::Policy::limited(10))
            .build()?;
        let resp = client.get(&url).send()?;
        let status = resp.status();
        if !status.is_success() {
            return Err(format!("HTTP {}: {}", status, url).into());
        }
        let bytes = resp.bytes()?;
        let mut f = std::fs::File::create(&out_path)?;
        f.write_all(&bytes)?;
        f.sync_all()?;

        println!("Downloaded graph ONNX -> {}", out_path.display());
        Ok(())
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    if let Err(e) = real::run() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
