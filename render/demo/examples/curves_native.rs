//! Curves demo: line segment, Bézier, Hermite, B-spline with gizmo.
//!
//! Run: `cargo run -p render-demo --example curves_native`

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    render_demo::run_native_curves()
}
