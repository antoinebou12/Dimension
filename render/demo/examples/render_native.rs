//! Native demo: cube, tetrahedron, cylinder with gizmo and picking.
//!
//! Controls: Use the Scene panel to select entities or change primitives.
//! Click on a shape in the viewport to select it; drag to orbit.

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    render_demo::run_native()
}
