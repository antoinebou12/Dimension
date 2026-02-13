//! Shader sources: default embedded WGSL and optional runtime override.
//!
//! Shaders live in `render/shaders/`: `scene.wgsl`, `ui.wgsl`, `compute_example.wgsl`.
//! Use [`ShaderSources`] for default embedded sources or [`ShaderConfig`] for custom WGSL.

/// Default embedded WGSL sources. Use these when no custom shader is provided.
pub struct ShaderSources;

impl ShaderSources {
    /// Scene pass: vertex + fragment (MVP transform, vertex color).
    /// Entry points: `vs_main`, `fs_main`.
    #[must_use]
    pub fn scene_vertex_fragment() -> &'static str {
        include_str!("../../shaders/scene.wgsl")
    }

    /// UI pass: vertex + fragment (screen-space ortho, alpha blending).
    /// Entry points: `vs_main`, `fs_main`.
    #[must_use]
    pub fn ui_vertex_fragment() -> &'static str {
        include_str!("../../shaders/ui.wgsl")
    }

    /// Example compute shader (template). Entry point: `cs_main`.
    /// See `render/shaders/compute_example.wgsl` and docs/render.md.
    #[must_use]
    pub fn compute_example() -> &'static str {
        include_str!("../../shaders/compute_example.wgsl")
    }

    /// Gizmo rotation rings: vertex + fragment (model_view, proj, ring alpha, active highlight).
    /// Entry points: `vs_main`, `fs_main`. Use with [`crate::backend::GizmoVertex`] layout.
    #[must_use]
    pub fn gizmo_vertex_fragment() -> &'static str {
        include_str!("../../shaders/gizmo.wgsl")
    }

    /// Slice plane: quad with checker pattern and Oren-Nayar lighting.
    /// Entry points: `vs_main`, `fs_main`. Use with [`crate::backend::SlicePlaneVertex`] layout.
    #[must_use]
    pub fn slice_plane_vertex_fragment() -> &'static str {
        include_str!("../../shaders/slice_plane.wgsl")
    }

    /// Grid cube: instanced unit cube per cell. Entry points: `vs_main`, `fs_main`.
    #[must_use]
    pub fn grid_cube_vertex_fragment() -> &'static str {
        include_str!("../../shaders/grid_cube.wgsl")
    }
}

/// Optional custom shader WGSL. When `Some`, overrides the default embedded source.
#[derive(Clone, Debug, Default)]
pub struct ShaderConfig {
    /// Custom scene pass WGSL (vertex + fragment).
    pub scene_wgsl: Option<String>,
    /// Custom UI pass WGSL (vertex + fragment).
    pub ui_wgsl: Option<String>,
}

impl ShaderConfig {
    /// Create empty config (uses embedded sources).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Scene WGSL to use, or default embedded.
    #[must_use]
    pub fn scene_wgsl(&self) -> &str {
        let default = ShaderSources::scene_vertex_fragment();
        self.scene_wgsl.as_deref().unwrap_or(default)
    }

    /// UI WGSL to use, or default embedded.
    #[must_use]
    pub fn ui_wgsl(&self) -> &str {
        let default = ShaderSources::ui_vertex_fragment();
        self.ui_wgsl.as_deref().unwrap_or(default)
    }

    /// Load scene WGSL from path (native only). On WASM, returns error.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_scene_from_path(&mut self, path: &std::path::Path) -> Result<(), std::io::Error> {
        self.scene_wgsl = Some(std::fs::read_to_string(path)?);
        Ok(())
    }

    /// Load UI WGSL from path (native only). On WASM, returns error.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_ui_from_path(&mut self, path: &std::path::Path) -> Result<(), std::io::Error> {
        self.ui_wgsl = Some(std::fs::read_to_string(path)?);
        Ok(())
    }
}
