//! render — 2D/3D rendering engine using wgpu and mathlib.
//!
//! Version: 0.1.0
//!
//! GPU-first rendering; mathlib is used for all CPU-side math (transforms, MVP, world matrix
//! chain, orthographic camera). Platform layer supports **wasm** (canvas, requestAnimationFrame)
//! and **native** (winit, pollster). Scene layer uses mathlib's `Tree` for the world graph
//! (parent/child hierarchy); with the `serde` feature, world and scene can be serialized to a
//! compact binary format. Backend is wgpu-only (pipelines, buffers, camera).
//!
//! See the repository **docs/render.md** for architecture, main types, and conventions.

mod error;

pub mod backend;
pub mod cull;
pub mod demo;
pub mod engine;
pub mod gizmo;
pub mod grid;
pub mod material;
pub mod pick;
pub mod platform;
pub mod scene;
#[cfg(feature = "serde")]
pub mod serialization;
pub mod slice_plane;
pub mod spatial;
pub mod ui;
pub mod view_mode;

pub use backend::{
    Camera, Camera3d, Framebuffer, GizmoVertex, GpuRenderer, Projection, ReadbackGuard,
    ReadbackLayout, RenderTarget, ShaderConfig, ShaderSources, Vertex,
};
pub use engine::{Engine, FrameStats, SceneLighting};
pub use error::RenderError;
pub use material::{Material, MaterialKind, MaterialRegistry, MaterialViews};
pub use platform::RunDemo;

pub use demo::{
    apply_scene_action, auto_select_first_entity, build_demo_scene, build_material_panel,
    build_scene_panel, build_stats_panel, format_primitive_tree, parse_primitive_name, Aabb2dIds,
    SceneAction, MATERIAL_WINDOW_ID, SCENE_WINDOW_ID, STATS_WINDOW_ID,
};
pub use gizmo::{
    gizmo_mesh, pick_gizmo_handle, GizmoAxis, GizmoMesh, GizmoMode, GIZMO_DEFAULT_SIZE,
    GIZMO_X_COLOR, GIZMO_Y_COLOR, GIZMO_Z_COLOR,
};
pub use grid::{build_grid_cube_instances, unit_cube_mesh, GridCubeDescriptor};
#[cfg(feature = "screenshot")]
pub use image::{ImageBuffer, Rgb};
pub use pick::pick_entity;
pub use platform::input_constants;
#[cfg(all(not(target_arch = "wasm32"), feature = "sdl3"))]
pub use platform::run_sdl3;
pub use platform::{run, run_demo};
pub use scene::{
    CurvePoint, EntityId, NodeData, Primitive, Primitive2D, Primitive3D, Transform, World,
};
#[cfg(feature = "serde")]
pub use serialization::{world_from_bytes, world_to_bytes};
pub use slice_plane::SlicePlane;
pub use ui::{
    vertical_stack, Button, Checkbox, ControlId, Label, LabelTextAlign, Rect, Slider, Theme,
    UiLayer, VerticalLayout, Window,
};
pub use view_mode::ViewMode;
