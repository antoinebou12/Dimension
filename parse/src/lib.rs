//! parse — Multi-format parsers for JSON, BJSON, TOON, XML, OBJ, MTL, BVH, glTF, PLY, images, archives.
//!
//! # Format support (feature-gated)
//!
//! | Feature | Formats |
//! |---------|---------|
//! | `json` | JSON |
//! | `bjson` | BJSON (bjson.org) |
//! | `toon` | TOON (Token-Oriented Object Notation) |
//! | `xml` | XML |
//! | `obj` | Wavefront OBJ |
//! | `mtl` | Wavefront MTL |
//! | `bvh` | BVH (motion capture) |
//! | `gltf` | glTF / GLB |
//! | `ply` | PLY |
//! | `stl` | STL (binary and ASCII) |
//! | `msh` | Gmsh MSH (nodes + tet elements) |
//! | `image` | PNG, JPEG |
//! | `archive` | ZIP, TAR |
//!
//! # WASM
//!
//! Use `--features wasm` or `--features "wasm simd"`. Do not use `parallel` with wasm32.

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

pub mod error;
pub(crate) mod parser;

pub use error::ParseError;

/// Shared 3D types (used by obj, gltf, ply).
pub mod mesh {
    use mathlib::{Point3, Vector3f};

    /// Vertex with position, normal, UV.
    #[derive(Clone, Debug)]
    pub struct Vertex {
        /// Position.
        pub position: Point3,
        /// Normal.
        pub normal: Vector3f,
        /// Texture coordinates.
        pub uv: (f32, f32),
        /// Optional tangent for normal mapping.
        pub tangent: Option<Vector3f>,
    }

    /// Triangle mesh.
    #[derive(Clone, Debug)]
    pub struct Mesh {
        /// Mesh name.
        pub name: String,
        /// Vertices (often expanded per-triangle).
        pub vertices: Vec<Vertex>,
        /// Optional material.
        pub material: Option<super::Material>,
    }

    impl Vertex {
        /// Creates a vertex.
        pub fn new(position: Point3, normal: Vector3f, uv: (f32, f32)) -> Self {
            Self {
                position,
                normal,
                uv,
                tangent: None,
            }
        }
    }
}

/// Material (MTL / glTF).
#[derive(Clone, Debug)]
pub struct Material {
    /// Material name.
    pub name: String,
    /// Ambient (rgba).
    pub ka: [f32; 4],
    /// Diffuse (rgba).
    pub kd: [f32; 4],
    /// Specular (rgba).
    pub ks: [f32; 4],
    /// Emissive (rgba).
    pub ke: [f32; 4],
    /// Shininess [0, 128].
    pub ns: f32,
    /// Index of refraction.
    pub ni: f32,
    /// Illumination model.
    pub illum: i32,
    /// PBR: roughness.
    pub pr: f32,
    /// PBR: metallic.
    pub pm: f32,
    /// PBR: sheen.
    pub ps: f32,
    /// PBR: clearcoat.
    pub pc: f32,
    /// PBR: clearcoat roughness.
    pub pcr: f32,
    /// Texture filenames (caller resolves paths).
    pub map_bump: Option<String>,
    /// Roughness texture map.
    pub map_roughness: Option<String>,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            name: "(Default)".to_string(),
            ka: [1.0; 4],
            kd: [1.0; 4],
            ks: [1.0; 4],
            ke: [0.0; 4],
            ns: 128.0,
            ni: 1.0,
            illum: 2,
            pr: 0.0,
            pm: 0.0,
            ps: 0.0,
            pc: 0.0,
            pcr: 0.0,
            map_bump: None,
            map_roughness: None,
        }
    }
}

#[cfg(feature = "json")]
pub mod json;

#[cfg(feature = "bjson")]
pub mod bjson;

#[cfg(feature = "toon")]
pub mod toon;

#[cfg(feature = "xml")]
pub mod xml;

#[cfg(feature = "obj")]
pub mod obj;

#[cfg(feature = "mtl")]
pub mod mtl;

#[cfg(feature = "bvh")]
pub mod bvh;

#[cfg(feature = "gltf")]
pub mod gltf;

#[cfg(feature = "ply")]
pub mod ply;

#[cfg(feature = "stl")]
pub mod stl;

#[cfg(feature = "msh")]
pub mod msh;

#[cfg(feature = "image")]
pub mod image;

#[cfg(feature = "archive")]
pub mod archive;
