//! Native demo: PBD particles with distance constraints.
//!
//! Controls: Left drag = orbit, Scroll = zoom.

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    physics_demo::run_native()
}
