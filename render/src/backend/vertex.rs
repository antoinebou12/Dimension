//! Vertex format for GPU rendering.

use bytemuck::{Pod, Zeroable};
use wgpu::VertexAttribute;
use wgpu::VertexFormat;
use wgpu::VertexStepMode;

/// Vertex for colored primitives. Position (vec3), UV (vec2), color (vec4).
/// For UI quads, rect_min/rect_max and corner_radius enable rounded-rect rendering; use 0 radius otherwise.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    /// Position (x, y, z); use z=0 for 2D.
    pub position: [f32; 3],
    /// UV coordinates.
    pub uv: [f32; 2],
    /// RGBA color.
    pub color: [f32; 4],
    /// Quad min corner (for rounded rect); use (0, 0) when not rounding.
    pub rect_min: [f32; 2],
    /// Quad max corner (for rounded rect); use (0, 0) when not rounding.
    pub rect_max: [f32; 2],
    /// Corner radius in pixels (0 = sharp corners).
    pub corner_radius: f32,
}

const VERTEX_OFFSET_0: u64 = 0;
const VERTEX_OFFSET_1: u64 = std::mem::size_of::<[f32; 3]>() as u64;
const VERTEX_OFFSET_2: u64 = VERTEX_OFFSET_1 + std::mem::size_of::<[f32; 2]>() as u64;
const VERTEX_OFFSET_3: u64 = VERTEX_OFFSET_2 + std::mem::size_of::<[f32; 4]>() as u64;
const VERTEX_OFFSET_4: u64 = VERTEX_OFFSET_3 + std::mem::size_of::<[f32; 2]>() as u64;
const VERTEX_OFFSET_5: u64 = VERTEX_OFFSET_4 + std::mem::size_of::<[f32; 2]>() as u64;

static VERTEX_ATTRIBUTES: [VertexAttribute; 6] = [
    VertexAttribute {
        offset: VERTEX_OFFSET_0,
        shader_location: 0,
        format: VertexFormat::Float32x3,
    },
    VertexAttribute {
        offset: VERTEX_OFFSET_1,
        shader_location: 1,
        format: VertexFormat::Float32x2,
    },
    VertexAttribute {
        offset: VERTEX_OFFSET_2,
        shader_location: 2,
        format: VertexFormat::Float32x4,
    },
    VertexAttribute {
        offset: VERTEX_OFFSET_3,
        shader_location: 3,
        format: VertexFormat::Float32x2,
    },
    VertexAttribute {
        offset: VERTEX_OFFSET_4,
        shader_location: 4,
        format: VertexFormat::Float32x2,
    },
    VertexAttribute {
        offset: VERTEX_OFFSET_5,
        shader_location: 5,
        format: VertexFormat::Float32,
    },
];

static SCENE_VERTEX_ATTRIBUTES: [VertexAttribute; 3] = [
    VertexAttribute {
        offset: VERTEX_OFFSET_0,
        shader_location: 0,
        format: VertexFormat::Float32x3,
    },
    VertexAttribute {
        offset: VERTEX_OFFSET_1,
        shader_location: 1,
        format: VertexFormat::Float32x2,
    },
    VertexAttribute {
        offset: VERTEX_OFFSET_2,
        shader_location: 2,
        format: VertexFormat::Float32x4,
    },
];

impl Vertex {
    /// Vertex buffer layout for wgpu.
    #[must_use]
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as u64,
            step_mode: VertexStepMode::Vertex,
            attributes: &VERTEX_ATTRIBUTES,
        }
    }

    /// Vertex buffer layout for scene pipelines (locations 0-2 only; excludes
    /// UI-only fields rect_min/rect_max/corner_radius to avoid colliding
    /// with SceneInstance locations 3+).
    #[must_use]
    pub fn scene_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as u64,
            step_mode: VertexStepMode::Vertex,
            attributes: &SCENE_VERTEX_ATTRIBUTES,
        }
    }
}

/// Vertex for gizmo rotation rings. Position, normal, color (RGB), component (axis id), texcoord for ring cross-section.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct GizmoVertex {
    /// Position (x, y, z).
    pub position: [f32; 3],
    /// Normal (view space used in shader; pass model-space).
    pub normal: [f32; 3],
    /// RGB color (shader uses vec3).
    pub color: [f32; 3],
    /// Component: (1,0,0) for X axis, (0,1,0) for Y, (0,0,1) for Z.
    pub component: [f32; 3],
    /// Texcoord for ring alpha: used as (pointRad, 0) or 2D cross-section.
    pub texcoord: [f32; 2],
}

/// Vertex for slice plane quad: position in local plane space (vec4, homogeneous).
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct SlicePlaneVertex {
    /// Position (x, y, z, w); use z = 0, w = 1 for a 2D quad in the plane.
    pub position: [f32; 4],
}

impl SlicePlaneVertex {
    /// Vertex buffer layout for wgpu (slice plane pipeline).
    #[must_use]
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<SlicePlaneVertex>() as u64,
            step_mode: VertexStepMode::Vertex,
            attributes: &[VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: VertexFormat::Float32x4,
            }],
        }
    }
}

/// Vertex for batched 3D points (position and color per vertex; 6 vertices per point quad).
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct PointVertex {
    /// Position (x, y, z) in world space.
    pub position: [f32; 3],
    /// RGBA color.
    pub color: [f32; 4],
}

impl PointVertex {
    /// Vertex buffer layout for the points overlay pipeline.
    #[must_use]
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<PointVertex>() as u64,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float32x3,
                },
                VertexAttribute {
                    offset: 12,
                    shader_location: 1,
                    format: VertexFormat::Float32x4,
                },
            ],
        }
    }
}

/// Unit-cube vertex for grid cube (position only, in [-1, 1]^3).
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct GridCubeVertex {
    pub position: [f32; 3],
}

impl GridCubeVertex {
    #[must_use]
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<GridCubeVertex>() as u64,
            step_mode: VertexStepMode::Vertex,
            attributes: &[VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: VertexFormat::Float32x3,
            }],
        }
    }
}

/// Per-instance data for scene primitives (replaces per-draw object uniform when instancing).
/// Layout matches WGSL: mvp (64) + model_view (64) + material_selected (16) + entity_color (16) = 160 bytes.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct SceneInstance {
    /// MVP matrix (column-major).
    pub mvp: [[f32; 4]; 4],
    /// Model-view matrix (column-major).
    pub model_view: [[f32; 4]; 4],
    /// Material mode (.x), selected flag (.y), padding (.z, .w).
    pub material_selected: [u32; 4],
    /// Entity color (RGBA).
    pub entity_color: [f32; 4],
}

impl SceneInstance {
    /// Vertex buffer layout for wgpu (instance step mode). Uses locations 6..=15 so they do not
    /// conflict with Vertex layout locations 3–5 (rect_min, rect_max, corner_radius) when both
    /// are bound in scene pipelines.
    #[must_use]
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        const SZ: u64 = std::mem::size_of::<SceneInstance>() as u64;
        const VEC4: u64 = 16;
        wgpu::VertexBufferLayout {
            array_stride: SZ,
            step_mode: VertexStepMode::Instance,
            attributes: &[
                VertexAttribute {
                    offset: 0,
                    shader_location: 6,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: VEC4,
                    shader_location: 7,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: VEC4 * 2,
                    shader_location: 8,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: VEC4 * 3,
                    shader_location: 9,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: VEC4 * 4,
                    shader_location: 10,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: VEC4 * 5,
                    shader_location: 11,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: VEC4 * 6,
                    shader_location: 12,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: VEC4 * 7,
                    shader_location: 13,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: VEC4 * 8,
                    shader_location: 14,
                    format: VertexFormat::Uint32x4,
                },
                VertexAttribute {
                    offset: VEC4 * 9,
                    shader_location: 15,
                    format: VertexFormat::Float32x4,
                },
            ],
        }
    }
}

/// Per-instance data for grid cube (cell center position and cell index).
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct GridCubeInstance {
    pub cell_position: [f32; 3],
    pub _pad: f32,
    pub cell_index: [u32; 3],
    pub _pad2: u32,
}

impl GridCubeInstance {
    #[must_use]
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<GridCubeInstance>() as u64,
            step_mode: VertexStepMode::Instance,
            attributes: &[
                VertexAttribute {
                    offset: 0,
                    shader_location: 1,
                    format: VertexFormat::Float32x3,
                },
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 4]>() as u64,
                    shader_location: 2,
                    format: VertexFormat::Uint32x3,
                },
            ],
        }
    }
}

impl GizmoVertex {
    /// Vertex buffer layout for wgpu (gizmo rotation pipeline).
    #[must_use]
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<GizmoVertex>() as u64,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float32x3,
                },
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as u64,
                    shader_location: 1,
                    format: VertexFormat::Float32x3,
                },
                VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 3]>() * 2) as u64,
                    shader_location: 2,
                    format: VertexFormat::Float32x3,
                },
                VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 3]>() * 3) as u64,
                    shader_location: 3,
                    format: VertexFormat::Float32x3,
                },
                VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 3]>() * 4) as u64,
                    shader_location: 4,
                    format: VertexFormat::Float32x2,
                },
            ],
        }
    }
}
