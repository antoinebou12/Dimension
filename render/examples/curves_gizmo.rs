//! Line and curve primitives demo with gizmo.
//!
//! Creates a scene with LineSegment, Bézier, Hermite, and B-spline entities; one is selected
//! so the transform gizmo is shown. Use the Scene panel to select other entities; orbit the
//! camera to view the curves.
//!
//! Run with: `cargo run -p render --example curves_gizmo`

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    render::run_demo(render::RunDemo::Curves)
}
