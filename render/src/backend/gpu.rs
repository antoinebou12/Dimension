//! wgpu renderer (GPU-first).

use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

use super::camera::Camera3d;
use super::framebuffer;
use super::framebuffer::Framebuffer;
use super::math_prep::{
    model_view_matrix, mvp_matrix, scene_frame_uniform_bytes, world_matrix,
    SCENE_FRAME_UNIFORM_SIZE,
};
use super::mesh::primitive_mesh;
use super::shaders::ShaderConfig;
use super::texture::create_texture_from_rgba;
use super::vertex::{
    GizmoVertex, GridCubeInstance, GridCubeVertex, SceneInstance, SlicePlaneVertex, Vertex,
};
use crate::cull::{primitive_aabb, world_aabb, Frustum};
use crate::error::RenderError;
use crate::gizmo::{gizmo_mesh, GizmoMesh, GizmoMode, GIZMO_DEFAULT_SIZE};
use crate::grid::{build_grid_cube_instances, unit_cube_mesh, GridCubeDescriptor};
use crate::material::{
    MaterialKind, MaterialRegistry, MaterialViews, MATERIAL_MODE_BLENDABLE, MATERIAL_MODE_STATIC,
    MATERIAL_MODE_UV_DIFFUSE, MATERIAL_MODE_VERTEX_COLOR,
};
use crate::scene::{EntityId, Primitive, World};
use crate::slice_plane::SlicePlane;
use crate::ui::UiLayer;
use crate::view_mode::ViewMode;
use mathlib::cg::matrix4f_identity;
use mathlib::colormap::scalar_to_rgb_viridis;
use mathlib::math3d::Matrix4f;
use wgpu::util::DeviceExt;

/// Colormap texture width (1D lookup).
const COLORMAP_SIZE: u32 = 256;

/// Default exposure for tone mapping.
const DEFAULT_EXPOSURE: f32 = 1.0;

/// Default gamma for tone mapping.
const DEFAULT_GAMMA: f32 = 2.2;

/// Gizmo uniform buffer size for bind group and buffer creation.
///
/// Must be 176: WGSL uniform layout rules (e.g. vec3 alignment 16) make the shader struct 176 bytes.
/// The Rust [`GizmoUniforms`] is 160 bytes; buffer and layout must use this WGSL size so validation
/// does not report "shader uses more bytes than layout's minBindingSize".
const GIZMO_UNIFORM_SIZE: usize = 176;

/// Gizmo cache entry: Arrows use scene pipeline (Vertex), Rings use gizmo-rotate pipeline (GizmoVertex).
enum GizmoCacheEntry {
    Arrows(wgpu::Buffer, wgpu::Buffer, u32),
    Rings(wgpu::Buffer, wgpu::Buffer, u32),
}

/// Gizmo shader uniforms (must match gizmo.wgsl layout).
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct GizmoUniforms {
    model_view: [[f32; 4]; 4],
    proj_matrix: [[f32; 4]; 4],
    disk_width_rel: f32,
    _pad: [f32; 3],
    active_axes: [f32; 3],
    _pad2: f32,
}

/// Slice plane uniforms (must match slice_plane.wgsl: view, proj, object, length_scale, transparency, color, grid_line_color).
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct SlicePlaneUniforms {
    view_matrix: [[f32; 4]; 4],
    proj_matrix: [[f32; 4]; 4],
    object_matrix: [[f32; 4]; 4],
    length_scale: f32,
    transparency: f32,
    _pad: [f32; 2],
    color: [f32; 3],
    _pad2: f32,
    grid_line_color: [f32; 3],
    _pad3: f32,
}

const SLICE_PLANE_UNIFORM_SIZE: usize = std::mem::size_of::<SlicePlaneUniforms>();

/// Grid cube uniforms (model_view, proj, grid_spacing, cube_size_factor).
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct GridCubeUniforms {
    model_view: [[f32; 4]; 4],
    proj_matrix: [[f32; 4]; 4],
    grid_spacing: [f32; 3],
    _pad: f32,
    cube_size_factor: f32,
    _pad2: [f32; 3],
}

/// Rust struct size (160); buffer and layout use [`GRID_CUBE_UNIFORM_SIZE_WGSL`].
#[allow(dead_code)]
const GRID_CUBE_UNIFORM_SIZE: usize = std::mem::size_of::<GridCubeUniforms>();
/// Grid cube uniform buffer size for bind group (WGSL layout with alignment = 176).
const GRID_CUBE_UNIFORM_SIZE_WGSL: usize = 176;

/// Create depth texture for scene pass. Returns `None` if width or height is zero.
fn create_depth_texture(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
) -> Option<wgpu::Texture> {
    if config.width == 0 || config.height == 0 {
        return None;
    }
    Some(device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Depth texture"),
        size: wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: DEPTH_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    }))
}

/// Depth format for the scene pass.
const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

/// Create readback texture for screenshot capture. Returns `None` if width or height is zero.
fn create_readback_texture(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
) -> Option<wgpu::Texture> {
    if config.width == 0 || config.height == 0 {
        return None;
    }
    Some(device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Readback texture"),
        size: wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: config.format,
        usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    }))
}

/// Render destination: either the surface (display + present) or an offscreen framebuffer.
pub enum RenderTarget<'a> {
    /// Render to the swap chain and present.
    Surface(&'a wgpu::Surface<'a>),
    /// Render to an offscreen framebuffer (no present).
    Framebuffer(&'a Framebuffer),
}

/// Internal: holds either surface output + views or framebuffer reference for the render path.
enum TargetViews<'a> {
    Surface {
        output: wgpu::SurfaceTexture,
        color_view: wgpu::TextureView,
        depth_view: wgpu::TextureView,
        width: u32,
        height: u32,
    },
    Framebuffer {
        fb: &'a Framebuffer,
    },
}

impl TargetViews<'_> {
    fn color_view(&self) -> &wgpu::TextureView {
        match self {
            TargetViews::Surface { color_view, .. } => color_view,
            TargetViews::Framebuffer { fb } => fb.view(),
        }
    }
    fn depth_view(&self) -> &wgpu::TextureView {
        match self {
            TargetViews::Surface { depth_view, .. } => depth_view,
            TargetViews::Framebuffer { fb } => fb.depth_view(),
        }
    }
    fn size(&self) -> (u32, u32) {
        match self {
            TargetViews::Surface { width, height, .. } => (*width, *height),
            TargetViews::Framebuffer { fb } => (fb.width(), fb.height()),
        }
    }
    fn output_texture_for_readback(&self) -> Option<&wgpu::Texture> {
        match self {
            TargetViews::Surface { output, .. } => Some(&output.texture),
            TargetViews::Framebuffer { .. } => None,
        }
    }
    fn take_surface_output(self) -> Option<wgpu::SurfaceTexture> {
        match self {
            TargetViews::Surface { output, .. } => Some(output),
            TargetViews::Framebuffer { .. } => None,
        }
    }
}

/// GpuRenderer: wgpu-based rendering.
pub struct GpuRenderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    /// Solid fill pipeline (TriangleList, Fill).
    pipeline: wgpu::RenderPipeline,
    /// Wireframe pipeline (TriangleList, Line). `None` on backends that don't support `POLYGON_MODE_LINE` (e.g. WebGPU).
    pipeline_wireframe: Option<wgpu::RenderPipeline>,
    /// Points pipeline (PointList). Same shader; draws vertices as points.
    pipeline_points: wgpu::RenderPipeline,
    /// Line list pipeline for curve/line primitives.
    pipeline_line: wgpu::RenderPipeline,
    /// Frame uniforms (view, proj, light, colormap_mode, exposure, gamma, ambient, selection_time). Written once per frame.
    frame_uniform_buffer: wgpu::Buffer,
    /// Incremented each frame; used to derive selection_time for selection pulse (blinking).
    frame_count: u32,
    /// Instance buffer for scene and gizmo (SceneInstance per instance). Grown as needed.
    scene_instance_buffer: wgpu::Buffer,
    /// Capacity of instance buffer in number of instances.
    scene_instance_capacity: usize,
    _colormap_texture: wgpu::Texture,
    _colormap_view: wgpu::TextureView,
    _colormap_sampler: wgpu::Sampler,
    _scene_bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
    material_bind_group_layout: wgpu::BindGroupLayout,
    _material_fallback_texture: wgpu::Texture,
    material_fallback_bind_group: wgpu::BindGroup,
    material_sampler: wgpu::Sampler,
    material_bind_group_cache: HashMap<String, wgpu::BindGroup>,
    config: wgpu::SurfaceConfiguration,
    /// Depth texture for 3D scene (recreated on resize).
    depth_texture: Option<wgpu::Texture>,
    /// Readback texture for screenshot capture (recreated on resize).
    readback_texture: Option<wgpu::Texture>,
    /// Mesh buffers per primitive type (lazily created): (vertex_buf, index_buf, index_count, vertex_count).
    mesh_cache: HashMap<Primitive, (wgpu::Buffer, wgpu::Buffer, u32, u32)>,
    /// Gizmo mesh buffers per mode (Arrows = scene pipeline, Rings = gizmo-rotate pipeline).
    gizmo_cache: HashMap<GizmoMode, GizmoCacheEntry>,
    /// Pipeline for rotation ring gizmo (GizmoVertex layout).
    pipeline_gizmo_rotate: wgpu::RenderPipeline,
    gizmo_uniform_buffer: wgpu::Buffer,
    gizmo_bind_group: wgpu::BindGroup,
    /// Slice plane pipeline and static quad (optional overlay).
    pipeline_slice_plane: wgpu::RenderPipeline,
    slice_plane_uniform_buffer: wgpu::Buffer,
    slice_plane_bind_group: wgpu::BindGroup,
    slice_plane_vertex_buffer: wgpu::Buffer,
    slice_plane_index_buffer: wgpu::Buffer,
    slice_plane_index_count: u32,
    /// Grid cube pipeline and unit cube mesh.
    pipeline_grid_cube: wgpu::RenderPipeline,
    grid_cube_uniform_buffer: wgpu::Buffer,
    grid_cube_bind_group: wgpu::BindGroup,
    grid_unit_cube_vertex_buffer: wgpu::Buffer,
    grid_unit_cube_index_buffer: wgpu::Buffer,
    grid_unit_cube_index_count: u32,
    /// Cached instance buffer for grid overlay (recreated when descriptor changes).
    grid_instance_buffer: Option<(wgpu::Buffer, u32)>,
    /// Points overlay pipeline and buffers (batched 3D points as quads).
    pipeline_points_overlay: wgpu::RenderPipeline,
    points_overlay_uniform_buffer: wgpu::Buffer,
    points_overlay_bind_group: wgpu::BindGroup,
    points_overlay_vertex_buffer: wgpu::Buffer,
    points_overlay_vertex_capacity: usize,
    /// Optional GPU timestamp query (scene pass duration). Set when `TIMESTAMP_QUERY` is supported.
    timestamp_query_set: Option<wgpu::QuerySet>,
    /// Buffer for resolving timestamp queries (2 × u64).
    timestamp_resolve_buffer: Option<wgpu::Buffer>,
    /// Staging buffers for timestamp readback (double-buffered); only present when timestamp queries are enabled.
    timestamp_staging_buffers: Option<[Arc<wgpu::Buffer>; 2]>,
    /// Index of the staging buffer to use for the next frame (alternates 0 and 1).
    timestamp_staging_index: u8,
    /// Last completed GPU time in ms; updated when a timestamp readback completes.
    gpu_time_ms: Arc<Mutex<Option<f32>>>,
    /// Pool of staging buffers for async pixel readback (COPY_DST | MAP_READ). Buffers are
    /// returned when the user drops the readback guard.
    readback_staging_pool: Arc<Mutex<Vec<(Arc<wgpu::Buffer>, u64)>>>,
}

/// Layout info for a pixel readback (padded row, dimensions, format).
#[derive(Clone, Copy, Debug)]
pub struct ReadbackLayout {
    /// Padded bytes per row (wgpu alignment).
    pub padded_bytes_per_row: u32,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Whether the texture format is BGRA (so caller can swap when producing RGB).
    pub is_bgra: bool,
}

/// Guard for an async pixel readback: holds the staging buffer until dropped.
/// Call [`Self::get_mapped_range`] to read data, then drop. On drop, unmaps and returns the buffer to the pool.
///
/// On WASM, ensure [`crate::Engine::poll_device`] (or equivalent) is called so the callback runs.
pub struct ReadbackGuard {
    buffer: Option<Arc<wgpu::Buffer>>,
    pool: Arc<Mutex<Vec<(Arc<wgpu::Buffer>, u64)>>>,
    size: u64,
    layout: ReadbackLayout,
}

impl ReadbackGuard {
    /// Returns the layout (dimensions, row stride, format hint).
    #[must_use]
    pub fn layout(&self) -> ReadbackLayout {
        self.layout
    }

    /// Returns the raw mapped bytes (padded rows, top-left origin). Call after the async callback
    /// has run with `Ok`. Then call [`Self::release`] or drop the guard to unmap and return the buffer.
    ///
    /// # Panics
    /// Panics if the buffer is not in a mapped state (e.g. callback reported error).
    pub fn get_mapped_range(&self) -> wgpu::BufferView {
        self.buffer
            .as_ref()
            .expect("guard already released")
            .slice(..)
            .get_mapped_range()
    }

    /// Unmaps the buffer and returns it to the staging pool. Idempotent; safe to call from [`Drop`].
    pub fn release(mut self) {
        if let Some(b) = self.buffer.take() {
            b.unmap();
            if let Ok(mut pool) = self.pool.lock() {
                pool.push((b, self.size));
            }
        }
    }
}

impl Drop for ReadbackGuard {
    fn drop(&mut self) {
        if let Some(b) = self.buffer.take() {
            b.unmap();
            if let Ok(mut pool) = self.pool.lock() {
                pool.push((b, self.size));
            }
        }
    }
}

fn scene_instance_from_mvp(
    mvp: &Matrix4f,
    model_view: &Matrix4f,
    material_mode: u32,
    entity_color: [f32; 4],
    selected: u32,
) -> SceneInstance {
    let m = mathlib::cg::matrix4f_to_array(mvp);
    let v = mathlib::cg::matrix4f_to_array(model_view);
    SceneInstance {
        mvp: [
            [m[0], m[1], m[2], m[3]],
            [m[4], m[5], m[6], m[7]],
            [m[8], m[9], m[10], m[11]],
            [m[12], m[13], m[14], m[15]],
        ],
        model_view: [
            [v[0], v[1], v[2], v[3]],
            [v[4], v[5], v[6], v[7]],
            [v[8], v[9], v[10], v[11]],
            [v[12], v[13], v[14], v[15]],
        ],
        material_selected: [material_mode, selected, 0, 0],
        entity_color,
    }
}

impl GpuRenderer {
    /// Reference to the device.
    #[must_use]
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    /// Reference to the queue.
    #[must_use]
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    /// Create renderer with surface, device, queue.
    ///
    /// # Errors
    /// Returns error if pipeline creation fails.
    pub fn new(
        device: wgpu::Device,
        queue: wgpu::Queue,
        config: wgpu::SurfaceConfiguration,
        shader_config: Option<&ShaderConfig>,
    ) -> Result<Self, RenderError> {
        debug_assert!(
            GIZMO_UNIFORM_SIZE >= std::mem::size_of::<GizmoUniforms>(),
            "GIZMO_UNIFORM_SIZE must be >= Rust struct size for WGSL layout"
        );
        debug_assert!(
            GRID_CUBE_UNIFORM_SIZE_WGSL >= std::mem::size_of::<GridCubeUniforms>(),
            "GRID_CUBE_UNIFORM_SIZE_WGSL must be >= Rust struct size for WGSL layout"
        );
        let shader = if shader_config.and_then(|c| c.scene_wgsl.as_ref()).is_some() {
            let wgsl = shader_config.unwrap().scene_wgsl();
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("shader"),
                source: wgpu::ShaderSource::Wgsl(wgsl.into()),
            })
        } else {
            device.create_shader_module(wgpu::include_wgsl!("../../shaders/scene.wgsl"))
        };

        let identity = matrix4f_identity();
        let init_frame = scene_frame_uniform_bytes(
            &identity,
            &identity,
            [0.577_350_27_f32; 3],
            0.0,
            None,
            0,
            DEFAULT_EXPOSURE,
            DEFAULT_GAMMA,
            0.45,
            0.0, // selection_time
        );
        let frame_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Scene frame uniform buffer"),
            contents: &init_frame,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let colormap_data: Vec<u8> = (0..COLORMAP_SIZE)
            .flat_map(|i| {
                let t = f64::from(i) / f64::from(COLORMAP_SIZE.saturating_sub(1));
                let [r, g, b] = scalar_to_rgb_viridis(t);
                [r, g, b, 255]
            })
            .collect();

        let (colormap_texture, colormap_view) = create_texture_from_rgba(
            &device,
            &queue,
            COLORMAP_SIZE,
            1,
            &colormap_data,
            Some("Colormap texture"),
        );

        let colormap_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Colormap sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Scene bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: std::num::NonZeroU64::new(
                            SCENE_FRAME_UNIFORM_SIZE as u64,
                        ),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        const INITIAL_INSTANCE_CAPACITY: usize = 128;
        let scene_instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Scene instance buffer"),
            size: (INITIAL_INSTANCE_CAPACITY * std::mem::size_of::<super::vertex::SceneInstance>())
                as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let scene_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Scene bind group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: frame_uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&colormap_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&colormap_sampler),
                },
            ],
        });

        let material_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Material Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        const WHITE_PX: [u8; 4] = [255, 255, 255, 255];
        let (material_fallback_texture, fallback_view) =
            create_texture_from_rgba(&device, &queue, 1, 1, &WHITE_PX, Some("material fallback"));
        let mat_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Material sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });
        let material_fallback_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Material fallback bind group"),
            layout: &material_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&fallback_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&fallback_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&fallback_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&fallback_view),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Sampler(&mat_sampler),
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout, &material_bind_group_layout],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[
                    Vertex::scene_layout(),
                    super::vertex::SceneInstance::layout(),
                ],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        // POLYGON_MODE_LINE is not supported on WebGPU; only create wireframe pipeline when the
        // feature is available (runtime check, more robust than cfg! for mixed target scenarios).
        let pipeline_wireframe = device
            .features()
            .contains(wgpu::Features::POLYGON_MODE_LINE)
            .then(|| {
                device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Wireframe Pipeline"),
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: Some("vs_main"),
                        buffers: &[
                            Vertex::scene_layout(),
                            super::vertex::SceneInstance::layout(),
                        ],
                        compilation_options: Default::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: Some("fs_main"),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: config.format,
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                        compilation_options: Default::default(),
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: None,
                        unclipped_depth: false,
                        polygon_mode: wgpu::PolygonMode::Line,
                        conservative: false,
                    },
                    depth_stencil: Some(wgpu::DepthStencilState {
                        format: DEPTH_FORMAT,
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::Less,
                        stencil: wgpu::StencilState::default(),
                        bias: wgpu::DepthBiasState::default(),
                    }),
                    multisample: wgpu::MultisampleState::default(),
                    multiview_mask: None,
                    cache: None,
                })
            });

        let pipeline_points = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Points Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[
                    Vertex::scene_layout(),
                    super::vertex::SceneInstance::layout(),
                ],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::PointList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let pipeline_line = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Line List Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[
                    Vertex::scene_layout(),
                    super::vertex::SceneInstance::layout(),
                ],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let gizmo_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Gizmo shader"),
            source: wgpu::ShaderSource::Wgsl(
                super::shaders::ShaderSources::gizmo_vertex_fragment().into(),
            ),
        });
        let identity = matrix4f_identity();
        let id_arr = mathlib::cg::matrix4f_to_array(&identity);
        let init_gizmo = GizmoUniforms {
            model_view: [
                id_arr[0..4].try_into().unwrap(),
                id_arr[4..8].try_into().unwrap(),
                id_arr[8..12].try_into().unwrap(),
                id_arr[12..16].try_into().unwrap(),
            ],
            proj_matrix: [
                id_arr[0..4].try_into().unwrap(),
                id_arr[4..8].try_into().unwrap(),
                id_arr[8..12].try_into().unwrap(),
                id_arr[12..16].try_into().unwrap(),
            ],
            disk_width_rel: 0.06,
            _pad: [0.0; 3],
            active_axes: [0.0, 0.0, 0.0],
            _pad2: 0.0,
        };
        let gizmo_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Gizmo uniform buffer"),
            size: GIZMO_UNIFORM_SIZE as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&gizmo_uniform_buffer, 0, bytemuck::bytes_of(&init_gizmo));
        let gizmo_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Gizmo bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: std::num::NonZeroU64::new(GIZMO_UNIFORM_SIZE as u64),
                    },
                    count: None,
                }],
            });
        let gizmo_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Gizmo bind group"),
            layout: &gizmo_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: gizmo_uniform_buffer.as_entire_binding(),
            }],
        });
        let gizmo_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Gizmo pipeline layout"),
                bind_group_layouts: &[&gizmo_bind_group_layout],
                ..Default::default()
            });
        let pipeline_gizmo_rotate =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Gizmo rotate pipeline"),
                layout: Some(&gizmo_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &gizmo_shader,
                    entry_point: Some("vs_main"),
                    buffers: &[GizmoVertex::layout()],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &gizmo_shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: None,
            });

        let slice_plane_quad: [SlicePlaneVertex; 4] = [
            SlicePlaneVertex {
                position: [-1.0, -1.0, 0.0, 1.0],
            },
            SlicePlaneVertex {
                position: [1.0, -1.0, 0.0, 1.0],
            },
            SlicePlaneVertex {
                position: [1.0, 1.0, 0.0, 1.0],
            },
            SlicePlaneVertex {
                position: [-1.0, 1.0, 0.0, 1.0],
            },
        ];
        let slice_plane_indices: [u16; 6] = [0, 1, 2, 2, 3, 0];
        let slice_plane_vertex_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Slice plane quad vertex buffer"),
                contents: bytemuck::cast_slice(&slice_plane_quad),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let slice_plane_index_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Slice plane quad index buffer"),
                contents: bytemuck::cast_slice(&slice_plane_indices),
                usage: wgpu::BufferUsages::INDEX,
            });
        let slice_plane_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Slice plane shader"),
            source: wgpu::ShaderSource::Wgsl(
                super::shaders::ShaderSources::slice_plane_vertex_fragment().into(),
            ),
        });
        let init_slice = SlicePlaneUniforms {
            view_matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            proj_matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            object_matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            length_scale: 1.0,
            transparency: 0.5,
            _pad: [0.0; 2],
            color: [0.9, 0.9, 0.95],
            _pad2: 0.0,
            grid_line_color: [0.7, 0.7, 0.75],
            _pad3: 0.0,
        };
        let slice_plane_uniform_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Slice plane uniform buffer"),
                contents: bytemuck::bytes_of(&init_slice),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
        let slice_plane_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Slice plane bind group layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: std::num::NonZeroU64::new(SLICE_PLANE_UNIFORM_SIZE as u64),
                },
                count: None,
            }],
        });
        let slice_plane_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Slice plane bind group"),
            layout: &slice_plane_bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: slice_plane_uniform_buffer.as_entire_binding(),
            }],
        });
        let slice_plane_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Slice plane pipeline layout"),
            bind_group_layouts: &[&slice_plane_bgl],
            ..Default::default()
        });
        let pipeline_slice_plane = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Slice plane pipeline"),
            layout: Some(&slice_plane_pl),
            vertex: wgpu::VertexState {
                module: &slice_plane_shader,
                entry_point: Some("vs_main"),
                buffers: &[SlicePlaneVertex::layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &slice_plane_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        const POINTS_OVERLAY_UNIFORM_SIZE: usize = 128;
        let points_overlay_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Points overlay shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/points.wgsl").into()),
        });
        let points_overlay_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Points overlay uniform buffer"),
            size: POINTS_OVERLAY_UNIFORM_SIZE as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let points_overlay_bgl =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Points overlay bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: std::num::NonZeroU64::new(
                            POINTS_OVERLAY_UNIFORM_SIZE as u64,
                        ),
                    },
                    count: None,
                }],
            });
        let points_overlay_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Points overlay bind group"),
            layout: &points_overlay_bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: points_overlay_uniform_buffer.as_entire_binding(),
            }],
        });
        let points_overlay_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Points overlay pipeline layout"),
            bind_group_layouts: &[&points_overlay_bgl],
            ..Default::default()
        });
        let pipeline_points_overlay =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Points overlay pipeline"),
                layout: Some(&points_overlay_pl),
                vertex: wgpu::VertexState {
                    module: &points_overlay_shader,
                    entry_point: Some("vs_main"),
                    buffers: &[super::vertex::PointVertex::layout()],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &points_overlay_shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: None,
            });
        const INITIAL_POINTS_VERTEX_CAPACITY: usize = 256;
        let points_overlay_vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Points overlay vertex buffer"),
            size: (INITIAL_POINTS_VERTEX_CAPACITY
                * std::mem::size_of::<super::vertex::PointVertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let (unit_cube_vertices, unit_cube_indices) = unit_cube_mesh();
        let grid_unit_cube_vertex_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Grid unit cube vertex buffer"),
                contents: bytemuck::cast_slice(&unit_cube_vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let grid_unit_cube_index_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Grid unit cube index buffer"),
                contents: bytemuck::cast_slice(&unit_cube_indices),
                usage: wgpu::BufferUsages::INDEX,
            });
        let grid_unit_cube_index_count = unit_cube_indices.len() as u32;
        let grid_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Grid cube shader"),
            source: wgpu::ShaderSource::Wgsl(
                super::shaders::ShaderSources::grid_cube_vertex_fragment().into(),
            ),
        });
        let init_grid = GridCubeUniforms {
            model_view: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            proj_matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            grid_spacing: [0.25, 0.25, 0.25],
            _pad: 0.0,
            cube_size_factor: 0.95,
            _pad2: [0.0; 3],
        };
        let grid_cube_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Grid cube uniform buffer"),
            size: GRID_CUBE_UNIFORM_SIZE_WGSL as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&grid_cube_uniform_buffer, 0, bytemuck::bytes_of(&init_grid));
        let grid_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Grid cube bind group layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: std::num::NonZeroU64::new(GRID_CUBE_UNIFORM_SIZE_WGSL as u64),
                },
                count: None,
            }],
        });
        let grid_cube_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Grid cube bind group"),
            layout: &grid_bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: grid_cube_uniform_buffer.as_entire_binding(),
            }],
        });
        let grid_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Grid cube pipeline layout"),
            bind_group_layouts: &[&grid_bgl],
            immediate_size: 0,
        });
        let pipeline_grid_cube = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Grid cube pipeline"),
            layout: Some(&grid_pl),
            vertex: wgpu::VertexState {
                module: &grid_shader,
                entry_point: Some("vs_main"),
                buffers: &[GridCubeVertex::layout(), GridCubeInstance::layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &grid_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let depth_texture = create_depth_texture(&device, &config);
        let readback_texture = create_readback_texture(&device, &config);

        // Timestamp query + staging buffers are disabled on WASM to avoid "Buffer is already mapped"
        // (map_async callbacks are deferred, so staging buffers can be reused before unmapped).
        let (timestamp_query_set, timestamp_resolve_buffer, timestamp_staging_buffers, gpu_time_ms) =
            if cfg!(not(target_arch = "wasm32"))
                && device.features().contains(wgpu::Features::TIMESTAMP_QUERY)
            {
                let query_set = device.create_query_set(&wgpu::QuerySetDescriptor {
                    label: Some("GPU timestamp query set"),
                    ty: wgpu::QueryType::Timestamp,
                    count: 2,
                });
                let resolve_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Timestamp resolve buffer"),
                    size: 2 * 8,
                    usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
                    mapped_at_creation: false,
                });
                let staging_buf_0 = Arc::new(device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Timestamp staging 0"),
                    size: 2 * 8,
                    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                    mapped_at_creation: false,
                }));
                let staging_buf_1 = Arc::new(device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Timestamp staging 1"),
                    size: 2 * 8,
                    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                    mapped_at_creation: false,
                }));
                (
                    Some(query_set),
                    Some(resolve_buffer),
                    Some([staging_buf_0, staging_buf_1]),
                    Arc::new(Mutex::new(None)),
                )
            } else {
                (None, None, None, Arc::new(Mutex::new(None)))
            };

        Ok(Self {
            device,
            queue,
            pipeline,
            pipeline_wireframe,
            pipeline_points,
            pipeline_line,
            frame_uniform_buffer,
            frame_count: 0,
            scene_instance_buffer,
            scene_instance_capacity: INITIAL_INSTANCE_CAPACITY,
            _colormap_texture: colormap_texture,
            _colormap_view: colormap_view,
            _colormap_sampler: colormap_sampler,
            _scene_bind_group_layout: bind_group_layout,
            bind_group: scene_bind_group,
            material_bind_group_layout,
            _material_fallback_texture: material_fallback_texture,
            material_fallback_bind_group,
            material_sampler: mat_sampler,
            material_bind_group_cache: HashMap::new(),
            config,
            depth_texture,
            readback_texture,
            mesh_cache: HashMap::new(),
            gizmo_cache: HashMap::new(),
            pipeline_gizmo_rotate,
            gizmo_uniform_buffer,
            gizmo_bind_group,
            pipeline_slice_plane,
            slice_plane_uniform_buffer,
            slice_plane_bind_group,
            slice_plane_vertex_buffer,
            slice_plane_index_buffer,
            slice_plane_index_count: 6,
            pipeline_points_overlay,
            points_overlay_uniform_buffer,
            points_overlay_bind_group,
            points_overlay_vertex_buffer,
            points_overlay_vertex_capacity: INITIAL_POINTS_VERTEX_CAPACITY,
            pipeline_grid_cube,
            grid_cube_uniform_buffer,
            grid_cube_bind_group,
            grid_unit_cube_vertex_buffer,
            grid_unit_cube_index_buffer,
            grid_unit_cube_index_count,
            grid_instance_buffer: None,
            timestamp_query_set,
            timestamp_resolve_buffer,
            timestamp_staging_buffers,
            timestamp_staging_index: 0,
            gpu_time_ms,
            readback_staging_pool: Arc::new(Mutex::new(Vec::new())),
        })
    }

    /// Last completed GPU time in milliseconds (from timestamp queries), if available.
    #[must_use]
    pub fn last_gpu_time_ms(&self) -> Option<f32> {
        self.gpu_time_ms.lock().unwrap().clone()
    }

    /// Creates an offscreen framebuffer (FBO) for render-to-texture and readback.
    ///
    /// Uses the surface config format when `format` is `None`. The FBO is fixed at creation;
    /// create a new one and drop the old when you need a different size.
    ///
    /// # Errors
    /// Returns error if width or height is zero or if texture creation fails.
    pub fn create_framebuffer(
        &self,
        width: u32,
        height: u32,
        format: Option<wgpu::TextureFormat>,
    ) -> Result<Framebuffer, RenderError> {
        framebuffer::create_framebuffer(
            &self.device,
            width,
            height,
            format.unwrap_or(self.config.format),
        )
    }

    /// Reads pixels from the readback texture into the provided buffer.
    ///
    /// Returns RGB data (3 bytes per pixel). Rows are ordered from bottom to top (OpenGL-style
    /// origin) for consistency with typical framebuffer readback APIs.
    ///
    /// # Panics
    /// Panics if the readback texture is not available (e.g. zero-sized viewport) or if the
    /// GPU readback fails.
    pub fn read_pixels(&self, out: &mut Vec<u8>, x: usize, y: usize, width: usize, height: usize) {
        let readback = self
            .readback_texture
            .as_ref()
            .expect("readback texture not available for screenshot");

        let bytes_per_pixel = 4; // RGBA or BGRA
        let unpadded_bytes_per_row = width * bytes_per_pixel;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize;
        let padded_bytes_per_row = unpadded_bytes_per_row.div_ceil(align) * align;
        let buffer_size = padded_bytes_per_row * height;

        let staging_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("screenshot_staging_buffer"),
            size: buffer_size as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("screenshot_copy_encoder"),
            });

        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: readback,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: x as u32,
                    y: y as u32,
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &staging_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bytes_per_row as u32),
                    rows_per_image: Some(height as u32),
                },
            },
            wgpu::Extent3d {
                width: width as u32,
                height: height as u32,
                depth_or_array_layers: 1,
            },
        );

        self.queue.submit(Some(encoder.finish()));

        let buffer_slice = staging_buffer.slice(..);
        let (tx, rx) = mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = tx.send(result);
        });

        let _ = self.device.poll(wgpu::PollType::wait_indefinitely());
        rx.recv().unwrap().expect("screenshot readback failed");

        let data = buffer_slice.get_mapped_range();

        let rgb_size = width * height * 3;
        out.clear();
        out.reserve(rgb_size);

        let is_bgra = matches!(
            self.config.format,
            wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb
        );

        // wgpu has origin at top-left; output rows bottom-to-top for OpenGL-style compatibility.
        for row in (0..height).rev() {
            let row_start = row * padded_bytes_per_row;
            for col in 0..width {
                let pixel_start = row_start + col * bytes_per_pixel;
                if is_bgra {
                    out.push(data[pixel_start + 2]); // R
                    out.push(data[pixel_start + 1]); // G
                    out.push(data[pixel_start]); // B
                } else {
                    out.push(data[pixel_start]); // R
                    out.push(data[pixel_start + 1]); // G
                    out.push(data[pixel_start + 2]); // B
                }
            }
        }

        drop(data);
        staging_buffer.unmap();
    }

    /// Reads pixels asynchronously from the readback texture (PBO-style).
    ///
    /// When the GPU copy and map complete, `callback` is invoked with `Ok(())` and a [`ReadbackGuard`]
    /// that provides [`ReadbackGuard::get_mapped_range`] to read raw RGBA bytes (padded rows, top-left origin).
    /// Call [`ReadbackGuard::release`] or drop the guard to unmap and return the buffer to the pool.
    /// On error, the callback receives `Err` and `None`; the staging buffer is still returned to the pool.
    ///
    /// On WASM, call [`crate::Engine::poll_device`] (or equivalent) after each frame so the callback runs;
    /// do not block the main thread.
    ///
    /// # Errors
    /// Returns error if the readback texture is not available.
    pub fn read_pixels_async(
        &self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        callback: impl FnOnce(Result<(), RenderError>, Option<ReadbackGuard>) + Send + 'static,
    ) -> Result<(), RenderError> {
        let readback = self.readback_texture.as_ref().ok_or_else(|| {
            RenderError::WgpuInit("readback texture not available for async readback".to_string())
        })?;

        let bytes_per_pixel = 4u32;
        let unpadded_bytes_per_row = width * bytes_per_pixel as usize;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize;
        let padded_bytes_per_row = unpadded_bytes_per_row.div_ceil(align) * align;
        let buffer_size = (padded_bytes_per_row * height) as u64;
        let is_bgra = matches!(
            self.config.format,
            wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb
        );
        let layout = ReadbackLayout {
            padded_bytes_per_row: padded_bytes_per_row as u32,
            width: width as u32,
            height: height as u32,
            is_bgra,
        };

        let pool = Arc::clone(&self.readback_staging_pool);
        let buffer: Arc<wgpu::Buffer> = {
            let mut p = pool
                .lock()
                .map_err(|_| RenderError::WgpuInit("readback pool lock poisoned".to_string()))?;
            let pos = p.iter().position(|(_, s)| *s >= buffer_size);
            if let Some(i) = pos {
                p.swap_remove(i).0
            } else {
                Arc::new(self.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("readback_staging"),
                    size: buffer_size,
                    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                    mapped_at_creation: false,
                }))
            }
        };

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("readback_async_encoder"),
            });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: readback,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: x as u32,
                    y: y as u32,
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bytes_per_row as u32),
                    rows_per_image: Some(height as u32),
                },
            },
            wgpu::Extent3d {
                width: width as u32,
                height: height as u32,
                depth_or_array_layers: 1,
            },
        );
        self.queue.submit(Some(encoder.finish()));

        let buffer_for_closure = Arc::clone(&buffer);
        buffer
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |result| {
                let (res, guard) = match result {
                    Ok(()) => (
                        Ok::<(), RenderError>(()),
                        Some(ReadbackGuard {
                            buffer: Some(Arc::clone(&buffer_for_closure)),
                            pool: Arc::clone(&pool),
                            size: buffer_size,
                            layout,
                        }),
                    ),
                    Err(e) => {
                        if let Ok(mut p) = pool.lock() {
                            p.push((buffer_for_closure, buffer_size));
                        }
                        (Err(RenderError::WgpuInit(e.to_string())), None)
                    }
                };
                callback(res, guard);
            });

        Ok(())
    }

    /// Reads pixels asynchronously from a framebuffer texture (e.g. after rendering to an FBO).
    ///
    /// Same as [`Self::read_pixels_async`] but uses `framebuffer` as the source instead of the
    /// internal readback texture. On WASM, call [`crate::Engine::poll_device`] so the callback runs.
    ///
    /// # Errors
    /// Returns error if the buffer or copy fails.
    pub fn read_pixels_async_from(
        &self,
        framebuffer: &Framebuffer,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        callback: impl FnOnce(Result<(), RenderError>, Option<ReadbackGuard>) + Send + 'static,
    ) -> Result<(), RenderError> {
        let texture = framebuffer.texture();
        let bytes_per_pixel = 4u32;
        let unpadded_bytes_per_row = width * bytes_per_pixel as usize;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize;
        let padded_bytes_per_row = unpadded_bytes_per_row.div_ceil(align) * align;
        let buffer_size = (padded_bytes_per_row * height) as u64;
        let is_bgra = matches!(
            framebuffer.format(),
            wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb
        );
        let layout = ReadbackLayout {
            padded_bytes_per_row: padded_bytes_per_row as u32,
            width: width as u32,
            height: height as u32,
            is_bgra,
        };

        let pool = Arc::clone(&self.readback_staging_pool);
        let buffer: Arc<wgpu::Buffer> = {
            let mut p = pool
                .lock()
                .map_err(|_| RenderError::WgpuInit("readback pool lock poisoned".to_string()))?;
            let pos = p.iter().position(|(_, s)| *s >= buffer_size);
            if let Some(i) = pos {
                p.swap_remove(i).0
            } else {
                Arc::new(self.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("readback_staging_from_fbo"),
                    size: buffer_size,
                    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                    mapped_at_creation: false,
                }))
            }
        };

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("readback_async_from_fbo_encoder"),
            });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: x as u32,
                    y: y as u32,
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bytes_per_row as u32),
                    rows_per_image: Some(height as u32),
                },
            },
            wgpu::Extent3d {
                width: width as u32,
                height: height as u32,
                depth_or_array_layers: 1,
            },
        );
        self.queue.submit(Some(encoder.finish()));

        let buffer_for_closure = Arc::clone(&buffer);
        buffer
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |result| {
                let (res, guard) = match result {
                    Ok(()) => (
                        Ok::<(), RenderError>(()),
                        Some(ReadbackGuard {
                            buffer: Some(Arc::clone(&buffer_for_closure)),
                            pool: Arc::clone(&pool),
                            size: buffer_size,
                            layout,
                        }),
                    ),
                    Err(e) => {
                        if let Ok(mut p) = pool.lock() {
                            p.push((buffer_for_closure, buffer_size));
                        }
                        (Err(RenderError::WgpuInit(e.to_string())), None)
                    }
                };
                callback(res, guard);
            });

        Ok(())
    }

    /// Ensure mesh buffers for this primitive exist in the cache; store index and vertex counts.
    fn ensure_mesh_cached(&mut self, prim: &Primitive) {
        if self.mesh_cache.contains_key(prim) {
            return;
        }
        let (vertices, indices) = primitive_mesh(prim);
        let vertex_count = vertices.len() as u32;
        let vertex_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Mesh vertex buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let index_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Mesh index buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            });
        let index_count = indices.len() as u32;
        self.mesh_cache.insert(
            *prim,
            (vertex_buffer, index_buffer, index_count, vertex_count),
        );
    }

    /// Resize surface and update config. Recreates the depth and readback textures.
    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.depth_texture = create_depth_texture(&self.device, &self.config);
            self.readback_texture = create_readback_texture(&self.device, &self.config);
        }
    }

    /// Ensure gizmo mesh for the given mode is cached. Arrows use scene pipeline; Rings use gizmo-rotate pipeline.
    fn ensure_gizmo_cached(&mut self, mode: GizmoMode) {
        if self.gizmo_cache.contains_key(&mode) {
            return;
        }
        let entry = match gizmo_mesh(mode, GIZMO_DEFAULT_SIZE) {
            GizmoMesh::Arrows(vertices, indices) => {
                let vb = self
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Gizmo arrows vertex buffer"),
                        contents: bytemuck::cast_slice(&vertices),
                        usage: wgpu::BufferUsages::VERTEX,
                    });
                let ib = self
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Gizmo arrows index buffer"),
                        contents: bytemuck::cast_slice(&indices),
                        usage: wgpu::BufferUsages::INDEX,
                    });
                GizmoCacheEntry::Arrows(vb, ib, indices.len() as u32)
            }
            GizmoMesh::Rings(vertices, indices) => {
                let vb = self
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Gizmo rings vertex buffer"),
                        contents: bytemuck::cast_slice(&vertices),
                        usage: wgpu::BufferUsages::VERTEX,
                    });
                let ib = self
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Gizmo rings index buffer"),
                        contents: bytemuck::cast_slice(&indices),
                        usage: wgpu::BufferUsages::INDEX,
                    });
                GizmoCacheEntry::Rings(vb, ib, indices.len() as u32)
            }
        };
        self.gizmo_cache.insert(mode, entry);
    }

    /// Ensure material bind group for `name` exists in cache; return it.
    fn ensure_material_bind_group(
        &mut self,
        materials: &MaterialRegistry,
        name: &str,
    ) -> &wgpu::BindGroup {
        if !self.material_bind_group_cache.contains_key(name) {
            if let Some(mat) = materials.get(name) {
                let mat_views = mat.views();
                let entries = match &mat_views {
                    MaterialViews::Static(view) => [
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::TextureView(view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: wgpu::BindingResource::TextureView(view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 4,
                            resource: wgpu::BindingResource::Sampler(&self.material_sampler),
                        },
                    ],
                    MaterialViews::Blendable { r, g, b, k } => [
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(r),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(g),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::TextureView(b),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: wgpu::BindingResource::TextureView(k),
                        },
                        wgpu::BindGroupEntry {
                            binding: 4,
                            resource: wgpu::BindingResource::Sampler(&self.material_sampler),
                        },
                    ],
                };
                let bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Material bind group"),
                    layout: &self.material_bind_group_layout,
                    entries: &entries,
                });
                self.material_bind_group_cache.insert(name.to_string(), bg);
            }
        }
        self.material_bind_group_cache
            .get(name)
            .unwrap_or(&self.material_fallback_bind_group)
    }

    /// Render to a target (surface or offscreen framebuffer).
    ///
    /// When target is [`RenderTarget::Surface`], the frame is copied to the internal readback
    /// texture (for screenshot) and presented. When target is [`RenderTarget::Framebuffer`],
    /// nothing is presented and the readback texture is not updated.
    ///
    /// # Errors
    /// Returns error on surface or render failure.
    pub fn render(
        &mut self,
        target: RenderTarget<'_>,
        world: &World,
        materials: &MaterialRegistry,
        camera: &mut impl Camera3d,
        ambient_intensity: f32,
        light_direction: [f32; 3],
        lighting_strength: f32,
        second_light: Option<[f32; 4]>,
        view_mode: ViewMode,
        selected_entity: Option<EntityId>,
        gizmo_mode: GizmoMode,
        slice_plane: Option<&SlicePlane>,
        grid_overlay: Option<&GridCubeDescriptor>,
        points: &[([f32; 3], [f32; 4], f32)],
        polylines: &[(Vec<[f32; 3]>, [f32; 4], f32)],
        ui: Option<&mut UiLayer>,
    ) -> Result<(), RenderError> {
        let target_views = match &target {
            RenderTarget::Surface(surface) => {
                let output = surface.get_current_texture().map_err(|e| {
                    use wgpu::SurfaceError;
                    match e {
                        SurfaceError::Outdated | SurfaceError::Lost => RenderError::SurfaceLost,
                        _ => RenderError::WgpuInit(e.to_string()),
                    }
                })?;
                let color_view = output
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                let depth_view = self
                    .depth_texture
                    .as_ref()
                    .expect("depth texture")
                    .create_view(&wgpu::TextureViewDescriptor::default());
                // Use actual surface texture size for viewport so we match the swapchain (handles resize races).
                let width = output.texture.width();
                let height = output.texture.height();
                TargetViews::Surface {
                    output,
                    color_view,
                    depth_view,
                    width,
                    height,
                }
            }
            RenderTarget::Framebuffer(fb) => TargetViews::Framebuffer { fb },
        };

        let (width, height) = target_views.size();
        camera.resize(width, height);
        let proj = camera.projection_matrix();
        let view_matrix = camera.view_matrix();
        let tree = world.tree();
        let n = tree.num_nodes();
        let mut world_matrices: Vec<Matrix4f> = (0..n).map(|_| matrix4f_identity()).collect();
        for id in world.entities_dfs() {
            let parent_world = world
                .parent(id)
                .map(|p| world_matrices[p.0].clone())
                .unwrap_or_else(matrix4f_identity);
            world_matrices[id.0] = world_matrix(tree, id.0, &parent_world);
        }

        let view_proj = &proj * &view_matrix;
        let frustum = Frustum::from_view_proj(&view_proj);
        let visible_entities: Vec<EntityId> = world
            .entities_dfs()
            .into_iter()
            .filter(|id| {
                let node = match world.get(*id) {
                    Some(n) => n,
                    None => return false,
                };
                let prim = match node.primitive {
                    Some(p) => p,
                    None => return false,
                };
                if prim.is_line_list() {
                    return true;
                }
                let model_aabb = primitive_aabb(&prim);
                let wb = world_aabb(&model_aabb, &world_matrices[id.0]);
                frustum.intersects_aabb(&wb)
            })
            .collect();

        #[cfg(target_arch = "wasm32")]
        {
            use std::sync::Once;
            use wasm_bindgen::JsValue;
            static LOG_VISIBLE_ONCE: Once = Once::new();
            LOG_VISIBLE_ONCE.call_once(|| {
                web_sys::console::log_1(&JsValue::from_str(&format!(
                    "render: first-frame visible_entities = {}",
                    visible_entities.len()
                )));
            });
        }

        let primitives: Vec<Primitive> = world
            .entities_dfs()
            .into_iter()
            .filter_map(|id| world.get(id).and_then(|node| node.primitive))
            .collect();
        for prim in &primitives {
            self.ensure_mesh_cached(prim);
        }
        if selected_entity.is_some() {
            self.ensure_gizmo_cached(gizmo_mode);
        }
        for id in world.entities_dfs() {
            if let Some(node) = world.get(id) {
                if node.primitive.is_some() {
                    let _ = self.ensure_material_bind_group(
                        materials,
                        node.material.as_deref().unwrap_or(""),
                    );
                }
            }
        }

        let queue = &self.queue;
        let cache = &self.mesh_cache;
        let gizmo_cache = &self.gizmo_cache;

        // Write frame uniforms once per frame.
        self.frame_count = self.frame_count.wrapping_add(1);
        let selection_time = self.frame_count as f32 * 0.016f32;
        let frame_bytes = scene_frame_uniform_bytes(
            &view_matrix,
            &proj,
            light_direction,
            lighting_strength,
            second_light,
            match view_mode {
                ViewMode::ColorMap => 1u32,
                ViewMode::Normals => 2u32,
                _ => 0u32,
            },
            DEFAULT_EXPOSURE,
            DEFAULT_GAMMA,
            ambient_intensity,
            selection_time,
        );
        queue.write_buffer(&self.frame_uniform_buffer, 0, &frame_bytes);

        // Count visible entities (frustum-culled) for object buffer capacity.
        // Add 1 when gizmo arrows are drawn (they use the scene pipeline and object buffer).
        let scene_draw_count = visible_entities.len();
        let _draw_count = scene_draw_count
            + if selected_entity.is_some() {
                if let Some(GizmoCacheEntry::Arrows(..)) = gizmo_cache.get(&gizmo_mode) {
                    1
                } else {
                    0
                }
            } else {
                0
            };
        let instance_capacity_required = scene_draw_count
            + if selected_entity.is_some()
                && matches!(
                    gizmo_cache.get(&gizmo_mode),
                    Some(GizmoCacheEntry::Arrows(..))
                )
            {
                1
            } else {
                0
            };
        if instance_capacity_required > self.scene_instance_capacity {
            let new_capacity = instance_capacity_required.next_power_of_two().max(1);
            self.scene_instance_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Scene instance buffer"),
                size: (new_capacity * std::mem::size_of::<super::vertex::SceneInstance>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.scene_instance_capacity = new_capacity;
        }

        let polyline_segment_count: usize = polylines
            .iter()
            .map(|(v, _, _)| v.len().saturating_sub(1))
            .sum();
        let points_overlay_capacity = points.len() * 6 + polyline_segment_count * 6;
        let mut points_overlay_vertices = Vec::with_capacity(points_overlay_capacity);
        for (pos, color, size) in points {
            let s = size / 2.0;
            let [px, py, pz] = *pos;
            let quad = [
                ([px - s, py - s, pz], *color),
                ([px + s, py - s, pz], *color),
                ([px + s, py + s, pz], *color),
                ([px - s, py - s, pz], *color),
                ([px + s, py + s, pz], *color),
                ([px - s, py + s, pz], *color),
            ];
            for (p, c) in quad {
                points_overlay_vertices.push(super::vertex::PointVertex {
                    position: p,
                    color: c,
                });
            }
        }
        for (vertices, color, width) in polylines {
            let half = width / 2.0;
            for seg in vertices.windows(2) {
                let (a, b) = (seg[0], seg[1]);
                let dx = b[0] - a[0];
                let dy = b[1] - a[1];
                let dz = b[2] - a[2];
                let len_sq = dx * dx + dy * dy + dz * dz;
                if len_sq < 1e-12_f32 {
                    continue;
                }
                let len = len_sq.sqrt();
                let (ux, uy, uz) = (dx / len, dy / len, dz / len);
                let (rx, ry, rz) = (
                    uy * 0.0 - uz * 1.0,
                    uz * 0.0 - ux * 0.0,
                    ux * 1.0 - uy * 0.0,
                );
                let r_len_sq = rx * rx + ry * ry + rz * rz;
                let (rx, ry, rz) = if r_len_sq < 1e-12_f32 {
                    (1.0_f32, 0.0, 0.0)
                } else {
                    let r_len = r_len_sq.sqrt();
                    (rx / r_len, ry / r_len, rz / r_len)
                };
                let c0 = [a[0] - rx * half, a[1] - ry * half, a[2] - rz * half];
                let c1 = [a[0] + rx * half, a[1] + ry * half, a[2] + rz * half];
                let c2 = [b[0] + rx * half, b[1] + ry * half, b[2] + rz * half];
                let c3 = [b[0] - rx * half, b[1] - ry * half, b[2] - rz * half];
                let quad = [
                    (c0, *color),
                    (c1, *color),
                    (c2, *color),
                    (c0, *color),
                    (c2, *color),
                    (c3, *color),
                ];
                for (p, c) in quad {
                    points_overlay_vertices.push(super::vertex::PointVertex {
                        position: p,
                        color: c,
                    });
                }
            }
        }
        if !points_overlay_vertices.is_empty() {
            let points_uniform: [f32; 16 * 2] = {
                let v = mathlib::cg::matrix4f_to_array(&view_matrix);
                let p = mathlib::cg::matrix4f_to_array(&proj);
                let mut out = [0.0_f32; 32];
                out[0..16].copy_from_slice(&v);
                out[16..32].copy_from_slice(&p);
                out
            };
            queue.write_buffer(
                &self.points_overlay_uniform_buffer,
                0,
                bytemuck::cast_slice(&points_uniform),
            );
            let needed = points_overlay_vertices.len();
            if needed > self.points_overlay_vertex_capacity {
                let new_cap = needed.next_power_of_two().max(1);
                self.points_overlay_vertex_buffer =
                    self.device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some("Points overlay vertex buffer"),
                        size: (new_cap * std::mem::size_of::<super::vertex::PointVertex>()) as u64,
                        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                        mapped_at_creation: false,
                    });
                self.points_overlay_vertex_capacity = new_cap;
            }
            queue.write_buffer(
                &self.points_overlay_vertex_buffer,
                0,
                bytemuck::cast_slice(&points_overlay_vertices),
            );
        }

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target_views.color_view(),
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.1,
                            b: 0.15,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: target_views.depth_view(),
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: self.timestamp_query_set.as_ref().map(|qs| {
                    wgpu::RenderPassTimestampWrites {
                        query_set: qs,
                        beginning_of_pass_write_index: Some(0),
                        end_of_pass_write_index: Some(1),
                    }
                }),
                occlusion_query_set: None,
                multiview_mask: None,
            });

            let (scene_pipeline, use_points) = match view_mode {
                ViewMode::Solid | ViewMode::ColorMap | ViewMode::Normals => (&self.pipeline, false),
                ViewMode::Wireframe => (
                    self.pipeline_wireframe.as_ref().unwrap_or(&self.pipeline),
                    false,
                ),
                ViewMode::VertexPoints => (&self.pipeline_points, true),
            };

            let mut batches: std::collections::HashMap<(Primitive, String), Vec<EntityId>> =
                std::collections::HashMap::new();
            for id in &visible_entities {
                let node = match world.get(*id) {
                    Some(n) => n,
                    None => continue,
                };
                let prim = match node.primitive {
                    Some(p) => p,
                    None => continue,
                };
                let mat_key = node.material.as_deref().unwrap_or("").to_string();
                batches.entry((prim, mat_key)).or_default().push(*id);
            }
            const INSTANCE_STRIDE: usize = std::mem::size_of::<SceneInstance>();
            let mut instance_offset: usize = 0;
            for ((prim, mat_key), ids) in &batches {
                let material_bg = self
                    .material_bind_group_cache
                    .get(mat_key.as_str())
                    .unwrap_or(&self.material_fallback_bind_group);
                let mut instances: Vec<SceneInstance> = Vec::with_capacity(ids.len());
                for id in ids {
                    let node = world.get(*id).unwrap();
                    let (material_mode, entity_color) = if let Some(ref mat_name) = node.material {
                        if let Some(mat) = materials.get(mat_name) {
                            let mode = match &mat.kind {
                                MaterialKind::Static(_) => MATERIAL_MODE_STATIC,
                                MaterialKind::Blendable(_) => MATERIAL_MODE_BLENDABLE,
                                MaterialKind::UvDiffuse(_) => MATERIAL_MODE_UV_DIFFUSE,
                            };
                            (mode, node.color)
                        } else {
                            (MATERIAL_MODE_VERTEX_COLOR, node.color)
                        }
                    } else {
                        (MATERIAL_MODE_VERTEX_COLOR, node.color)
                    };
                    let world_mat = &world_matrices[id.0];
                    let mvp = mvp_matrix(world_mat, &view_matrix, &proj);
                    let model_view = model_view_matrix(world_mat, &view_matrix);
                    let selected = if selected_entity == Some(*id) {
                        1u32
                    } else {
                        0u32
                    };
                    instances.push(scene_instance_from_mvp(
                        &mvp,
                        &model_view,
                        material_mode,
                        entity_color,
                        selected,
                    ));
                }
                let offset_bytes = (instance_offset * INSTANCE_STRIDE) as u64;
                let size_bytes = (instances.len() * INSTANCE_STRIDE) as u64;
                queue.write_buffer(
                    &self.scene_instance_buffer,
                    offset_bytes,
                    bytemuck::cast_slice(&instances),
                );
                let (vb, ib, index_count, vertex_count) = cache.get(prim).expect("mesh cached");
                render_pass.set_vertex_buffer(0, vb.slice(..));
                render_pass.set_vertex_buffer(
                    1,
                    self.scene_instance_buffer
                        .slice(offset_bytes..offset_bytes + size_bytes),
                );
                render_pass.set_bind_group(0, &self.bind_group, &[]);
                render_pass.set_bind_group(1, material_bg, &[]);
                let instance_count = instances.len() as u32;
                if prim.is_line_list() {
                    render_pass.set_pipeline(&self.pipeline_line);
                    render_pass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint16);
                    render_pass.draw_indexed(0..*index_count, 0, 0..instance_count);
                } else {
                    render_pass.set_pipeline(scene_pipeline);
                    if use_points {
                        render_pass.draw(0..*vertex_count, 0..instance_count);
                    } else {
                        render_pass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint16);
                        render_pass.draw_indexed(0..*index_count, 0, 0..instance_count);
                    }
                }
                instance_offset += instances.len();
            }

            if !points_overlay_vertices.is_empty() {
                render_pass.set_pipeline(&self.pipeline_points_overlay);
                render_pass.set_bind_group(0, &self.points_overlay_bind_group, &[]);
                render_pass.set_vertex_buffer(
                    0,
                    self.points_overlay_vertex_buffer.slice(
                        ..(points_overlay_vertices.len()
                            * std::mem::size_of::<super::vertex::PointVertex>())
                            as u64,
                    ),
                );
                render_pass.draw(0..(points_overlay_vertices.len() as u32), 0..1);
            }
        }

        if let Some(slice_plane) = slice_plane {
            let view_arr = mathlib::cg::matrix4f_to_array(&view_matrix);
            let proj_arr = mathlib::cg::matrix4f_to_array(&proj);
            let obj_arr = mathlib::cg::matrix4f_to_array(&slice_plane.object_matrix);
            let slice_uniform = SlicePlaneUniforms {
                view_matrix: [
                    view_arr[0..4].try_into().unwrap(),
                    view_arr[4..8].try_into().unwrap(),
                    view_arr[8..12].try_into().unwrap(),
                    view_arr[12..16].try_into().unwrap(),
                ],
                proj_matrix: [
                    proj_arr[0..4].try_into().unwrap(),
                    proj_arr[4..8].try_into().unwrap(),
                    proj_arr[8..12].try_into().unwrap(),
                    proj_arr[12..16].try_into().unwrap(),
                ],
                object_matrix: [
                    obj_arr[0..4].try_into().unwrap(),
                    obj_arr[4..8].try_into().unwrap(),
                    obj_arr[8..12].try_into().unwrap(),
                    obj_arr[12..16].try_into().unwrap(),
                ],
                length_scale: slice_plane.length_scale,
                transparency: slice_plane.transparency,
                _pad: [0.0; 2],
                color: slice_plane.color,
                _pad2: 0.0,
                grid_line_color: slice_plane.grid_line_color,
                _pad3: 0.0,
            };
            queue.write_buffer(
                &self.slice_plane_uniform_buffer,
                0,
                bytemuck::bytes_of(&slice_uniform),
            );
            let mut slice_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Slice plane pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target_views.color_view(),
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: target_views.depth_view(),
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            slice_pass.set_pipeline(&self.pipeline_slice_plane);
            slice_pass.set_bind_group(0, &self.slice_plane_bind_group, &[]);
            slice_pass.set_vertex_buffer(0, self.slice_plane_vertex_buffer.slice(..));
            slice_pass.set_index_buffer(
                self.slice_plane_index_buffer.slice(..),
                wgpu::IndexFormat::Uint16,
            );
            slice_pass.draw_indexed(0..self.slice_plane_index_count, 0, 0..1);
        }

        if let Some(descriptor) = grid_overlay {
            let instances = build_grid_cube_instances(descriptor);
            let instance_count = instances.len() as u32;
            let need_new_buffer = self
                .grid_instance_buffer
                .as_ref()
                .map_or(true, |(_, c)| *c != instance_count);
            if need_new_buffer && !instances.is_empty() {
                let buf = self
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Grid instance buffer"),
                        contents: bytemuck::cast_slice(&instances),
                        usage: wgpu::BufferUsages::VERTEX,
                    });
                self.grid_instance_buffer = Some((buf, instance_count));
            } else if !instances.is_empty() {
                if let Some((ref buf, _)) = self.grid_instance_buffer {
                    queue.write_buffer(buf, 0, bytemuck::cast_slice(&instances));
                }
            }
            if let Some((ref inst_buf, count)) = self.grid_instance_buffer {
                if count > 0 {
                    let mv_arr = mathlib::cg::matrix4f_to_array(&view_matrix);
                    let proj_arr = mathlib::cg::matrix4f_to_array(&proj);
                    let grid_uniform = GridCubeUniforms {
                        model_view: [
                            mv_arr[0..4].try_into().unwrap(),
                            mv_arr[4..8].try_into().unwrap(),
                            mv_arr[8..12].try_into().unwrap(),
                            mv_arr[12..16].try_into().unwrap(),
                        ],
                        proj_matrix: [
                            proj_arr[0..4].try_into().unwrap(),
                            proj_arr[4..8].try_into().unwrap(),
                            proj_arr[8..12].try_into().unwrap(),
                            proj_arr[12..16].try_into().unwrap(),
                        ],
                        grid_spacing: descriptor.spacing,
                        _pad: 0.0,
                        cube_size_factor: descriptor.cube_size_factor,
                        _pad2: [0.0; 3],
                    };
                    queue.write_buffer(
                        &self.grid_cube_uniform_buffer,
                        0,
                        bytemuck::bytes_of(&grid_uniform),
                    );
                    let mut grid_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Grid cube pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: target_views.color_view(),
                            resolve_target: None,
                            depth_slice: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                            view: target_views.depth_view(),
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            }),
                            stencil_ops: None,
                        }),
                        timestamp_writes: None,
                        occlusion_query_set: None,
                        multiview_mask: None,
                    });
                    grid_pass.set_pipeline(&self.pipeline_grid_cube);
                    grid_pass.set_bind_group(0, &self.grid_cube_bind_group, &[]);
                    grid_pass.set_vertex_buffer(0, self.grid_unit_cube_vertex_buffer.slice(..));
                    grid_pass.set_vertex_buffer(1, inst_buf.slice(..));
                    grid_pass.set_index_buffer(
                        self.grid_unit_cube_index_buffer.slice(..),
                        wgpu::IndexFormat::Uint16,
                    );
                    grid_pass.draw_indexed(0..self.grid_unit_cube_index_count, 0, 0..count);
                }
            }
        } else {
            self.grid_instance_buffer = None;
        }

        if let Some(id) = selected_entity {
            if id.0 < world_matrices.len() && world.get(id).is_some_and(|n| n.active) {
                if let Some(entry) = gizmo_cache.get(&gizmo_mode) {
                    let (vb, ib, count) = match entry {
                        GizmoCacheEntry::Arrows(v, i, c) => (v, i, c),
                        GizmoCacheEntry::Rings(v, i, c) => (v, i, c),
                    };
                    let world_mat = &world_matrices[id.0];
                    let model_view = model_view_matrix(world_mat, &view_matrix);
                    let mut gizmo_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Gizmo pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: target_views.color_view(),
                            resolve_target: None,
                            depth_slice: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                            view: target_views.depth_view(),
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            }),
                            stencil_ops: None,
                        }),
                        timestamp_writes: None,
                        occlusion_query_set: None,
                        multiview_mask: None,
                    });
                    match entry {
                        GizmoCacheEntry::Arrows(..) => {
                            let mvp = mvp_matrix(world_mat, &view_matrix, &proj);
                            let gizmo_instance = scene_instance_from_mvp(
                                &mvp,
                                &model_view,
                                MATERIAL_MODE_VERTEX_COLOR,
                                [1.0, 1.0, 1.0, 1.0],
                                0u32, // not selected (gizmo)
                            );
                            const INSTANCE_STRIDE_G: usize = std::mem::size_of::<SceneInstance>();
                            let gizmo_offset = (scene_draw_count * INSTANCE_STRIDE_G) as u64;
                            queue.write_buffer(
                                &self.scene_instance_buffer,
                                gizmo_offset,
                                bytemuck::bytes_of(&gizmo_instance),
                            );
                            gizmo_pass.set_pipeline(&self.pipeline);
                            gizmo_pass.set_bind_group(0, &self.bind_group, &[]);
                            gizmo_pass.set_bind_group(1, &self.material_fallback_bind_group, &[]);
                            gizmo_pass.set_vertex_buffer(
                                1,
                                self.scene_instance_buffer
                                    .slice(gizmo_offset..gizmo_offset + INSTANCE_STRIDE_G as u64),
                            );
                        }
                        GizmoCacheEntry::Rings(..) => {
                            let proj_arr = mathlib::cg::matrix4f_to_array(&proj);
                            let mv_arr = mathlib::cg::matrix4f_to_array(&model_view);
                            let gizmo_uniform = GizmoUniforms {
                                model_view: [
                                    mv_arr[0..4].try_into().unwrap(),
                                    mv_arr[4..8].try_into().unwrap(),
                                    mv_arr[8..12].try_into().unwrap(),
                                    mv_arr[12..16].try_into().unwrap(),
                                ],
                                proj_matrix: [
                                    proj_arr[0..4].try_into().unwrap(),
                                    proj_arr[4..8].try_into().unwrap(),
                                    proj_arr[8..12].try_into().unwrap(),
                                    proj_arr[12..16].try_into().unwrap(),
                                ],
                                disk_width_rel: 0.06,
                                _pad: [0.0; 3],
                                active_axes: [0.0, 0.0, 0.0],
                                _pad2: 0.0,
                            };
                            queue.write_buffer(
                                &self.gizmo_uniform_buffer,
                                0,
                                bytemuck::bytes_of(&gizmo_uniform),
                            );
                            gizmo_pass.set_pipeline(&self.pipeline_gizmo_rotate);
                            gizmo_pass.set_bind_group(0, &self.gizmo_bind_group, &[]);
                        }
                    }
                    gizmo_pass.set_vertex_buffer(0, vb.slice(..));
                    gizmo_pass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint16);
                    gizmo_pass.draw_indexed(0..*count, 0, 0..1);
                }
            }
        }

        if let Some(ui_layer) = ui {
            ui_layer.prepare_render(self.device(), self.queue());
            let mut ui_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("UI pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target_views.color_view(),
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            ui_pass.set_viewport(0.0, 0.0, width as f32, height as f32, 0.0, 1.0);
            ui_layer.encode_into_pass(&mut ui_pass);
        }

        let timestamp_buf_ix =
            if let (Some(ref qs), Some(ref resolve_buf), Some(ref staging_buffers)) = (
                &self.timestamp_query_set,
                &self.timestamp_resolve_buffer,
                &self.timestamp_staging_buffers,
            ) {
                let buf_ix = self.timestamp_staging_index as usize;
                encoder.resolve_query_set(qs, 0..2, resolve_buf, 0);
                encoder.copy_buffer_to_buffer(resolve_buf, 0, &staging_buffers[buf_ix], 0, 16);
                self.timestamp_staging_index = 1 - self.timestamp_staging_index;
                Some((Arc::clone(&staging_buffers[buf_ix]), buf_ix))
            } else {
                None
            };

        // Copy frame to readback texture for screenshot capture (surface only).
        if let (Some(ref readback), Some(src_texture)) = (
            &self.readback_texture,
            target_views.output_texture_for_readback(),
        ) {
            let (width, height) = target_views.size();
            if width > 0 && height > 0 {
                encoder.copy_texture_to_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: src_texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    wgpu::TexelCopyTextureInfo {
                        texture: readback,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    wgpu::Extent3d {
                        width,
                        height,
                        depth_or_array_layers: 1,
                    },
                );
            }
        }

        self.queue.submit(Some(encoder.finish()));

        if let Some(output) = target_views.take_surface_output() {
            output.present();
        }

        if let Some((staging_buf, _)) = timestamp_buf_ix {
            let gpu_time = Arc::clone(&self.gpu_time_ms);
            let period = self.queue.get_timestamp_period();
            let staging_buf_clone = Arc::clone(&staging_buf);
            staging_buf
                .slice(..)
                .map_async(wgpu::MapMode::Read, move |result| {
                    if result.is_ok() {
                        let mapped = staging_buf_clone.slice(..).get_mapped_range();
                        let bytes = mapped.as_ref();
                        if bytes.len() >= 16 {
                            let start = u64::from_ne_bytes(bytes[0..8].try_into().unwrap());
                            let end = u64::from_ne_bytes(bytes[8..16].try_into().unwrap());
                            let ns = (end - start) as f64 * period as f64;
                            let ms = (ns / 1e6) as f32;
                            *gpu_time.lock().unwrap() = Some(ms);
                        }
                    }
                });
        }

        Ok(())
    }
}
