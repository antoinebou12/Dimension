//! Native geometry demo (winit).
//!
//! Controls: Left drag = orbit, Scroll = zoom.

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    geometry_demo::run_native()
}
