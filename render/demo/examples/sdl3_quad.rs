//! SDL3 demo: fullscreen quad via wgpu.
//!
//! Run with: `cargo run -p render-demo --example sdl3_quad --features sdl3`

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    render::run_sdl3()
}
