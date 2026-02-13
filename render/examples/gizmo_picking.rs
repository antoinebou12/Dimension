//! Gizmo and picking demo: click on a shape to select it and show the transform gizmo.
//!
//! - Use the Scene panel to select an entity or change its primitive.
//! - Hold **Shift** and click on a shape in the viewport to select it (picking); the gizmo appears at the selection.
//! - Drag in the viewport (not on UI) to orbit the camera.
//! - Hold Ctrl and drag to move the camera only (orbit or pan); the gizmo is hidden and picking
//!   is disabled while Ctrl is held.
//!
//! View mode (solid, wireframe, vertex points) and gizmo mode (translate, rotate, scale) can be
//! set programmatically via [`Engine::set_view_mode`] and [`Engine::set_gizmo_mode`].

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    render::run()
}
