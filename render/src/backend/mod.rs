//! Backend: wgpu rendering, math prep.

mod camera;
mod framebuffer;
mod gpu;
pub mod math_prep;
mod mesh;
mod shaders;
mod texture;
mod vertex;

pub use camera::{Camera, Camera3d, Projection};
pub use framebuffer::{create_framebuffer, Framebuffer};
pub use gpu::{GpuRenderer, ReadbackGuard, ReadbackLayout, RenderTarget};
pub use mesh::primitive_mesh;
pub use shaders::{ShaderConfig, ShaderSources};
#[cfg(feature = "material")]
pub use texture::create_texture_from_image;
pub use texture::{create_texture_from_rgba, create_texture_from_rgba_with_mipmaps};
pub use vertex::{
    GizmoVertex, GridCubeInstance, GridCubeVertex, PointVertex, SceneInstance, SlicePlaneVertex,
    Vertex,
};
