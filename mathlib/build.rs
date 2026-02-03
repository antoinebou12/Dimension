//! Build script: rejects wasm32 + parallel feature (rayon is not wasm32-compatible).

fn main() {
    let arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    let features = std::env::var("CARGO_CFG_FEATURES").unwrap_or_default();
    if arch == "wasm32" && features.split_whitespace().any(|f| f == "parallel") {
        eprintln!(
            "error: the `parallel` feature is not supported for target wasm32; \
             use `--features wasm` or `--features wasm,simd` only."
        );
        std::process::exit(1);
    }
}
