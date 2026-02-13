//! Material system for Polyscope-style matcaps.
//!
//! Supports static materials (single texture) and blendable materials (four R/G/B/K basis textures).

use std::collections::HashMap;
use wgpu::Texture;
use wgpu::TextureView;

/// Material mode for shader branching: 0 = vertex color, 1 = static, 2 = blendable, 3 = UV diffuse.
pub(crate) const MATERIAL_MODE_VERTEX_COLOR: u32 = 0;
pub(crate) const MATERIAL_MODE_STATIC: u32 = 1;
pub(crate) const MATERIAL_MODE_BLENDABLE: u32 = 2;
pub(crate) const MATERIAL_MODE_UV_DIFFUSE: u32 = 3;

/// Static material: single matcap texture. Ignores entity color.
#[derive(Debug)]
pub struct StaticMaterial {
    /// Owned texture (keeps view valid).
    pub texture: Texture,
}

/// Blendable material: four basis textures (R, G, B, K). Entity color (r,g,b) drives blending:
/// `output = r*R + g*G + b*B + (1-r-g-b)*K`
#[derive(Debug)]
pub struct BlendableMaterial {
    /// Red basis texture.
    pub r: Texture,
    /// Green basis texture.
    pub g: Texture,
    /// Blue basis texture.
    pub b: Texture,
    /// Black basis texture.
    pub k: Texture,
}

/// Material kind: static matcap, blendable, or UV-mapped diffuse.
#[derive(Debug)]
pub enum MaterialKind {
    /// Single matcap; ignores entity color. Uses view-space normal for UV.
    Static(StaticMaterial),
    /// Four basis textures; entity color (r,g,b) blends them.
    Blendable(BlendableMaterial),
    /// Single texture sampled at vertex UV (e.g. procedural); bilinear via sampler.
    UvDiffuse(StaticMaterial),
}

/// Named material with optional RGB blending support.
#[derive(Debug)]
pub struct Material {
    /// Display name.
    pub name: String,
    /// Static or blendable.
    pub kind: MaterialKind,
    /// If true, material supports RGB color blending (blendable materials).
    pub supports_rgb: bool,
}

/// Registry of materials by name. Lives on [`crate::engine::Engine`].
pub type MaterialRegistry = HashMap<String, Material>;

impl Material {
    /// Create a static material. Takes ownership of the texture.
    #[must_use]
    pub fn static_mat(name: impl Into<String>, texture: Texture) -> Self {
        Self {
            name: name.into(),
            kind: MaterialKind::Static(StaticMaterial { texture }),
            supports_rgb: false,
        }
    }

    /// Create a blendable material. Takes ownership of the four textures.
    #[must_use]
    pub fn blendable(
        name: impl Into<String>,
        r: Texture,
        g: Texture,
        b: Texture,
        k: Texture,
    ) -> Self {
        Self {
            name: name.into(),
            kind: MaterialKind::Blendable(BlendableMaterial { r, g, b, k }),
            supports_rgb: true,
        }
    }

    /// Create a UV-mapped diffuse material. Single texture sampled at vertex UV (bilinear).
    #[must_use]
    pub fn uv_diffuse_mat(name: impl Into<String>, texture: Texture) -> Self {
        Self {
            name: name.into(),
            kind: MaterialKind::UvDiffuse(StaticMaterial { texture }),
            supports_rgb: false,
        }
    }

    /// Get the texture view(s) for binding. Static and UvDiffuse return one view; blendable returns four (r,g,b,k).
    pub fn views(&self) -> MaterialViews {
        match &self.kind {
            MaterialKind::Static(s) => {
                MaterialViews::Static(s.texture.create_view(&Default::default()))
            }
            MaterialKind::UvDiffuse(s) => {
                MaterialViews::Static(s.texture.create_view(&Default::default()))
            }
            MaterialKind::Blendable(b) => MaterialViews::Blendable {
                r: b.r.create_view(&Default::default()),
                g: b.g.create_view(&Default::default()),
                b: b.b.create_view(&Default::default()),
                k: b.k.create_view(&Default::default()),
            },
        }
    }
}

/// Borrowed texture views for a material (used during render).
pub enum MaterialViews {
    /// Single texture view.
    Static(TextureView),
    /// Four basis texture views.
    Blendable {
        r: TextureView,
        g: TextureView,
        b: TextureView,
        k: TextureView,
    },
}

/// Register the built-in "flat" material (1x1 white, preserves vertex color).
#[cfg(feature = "material")]
pub fn register_builtin_flat(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    registry: &mut MaterialRegistry,
) {
    use crate::backend::create_texture_from_rgba;
    const WHITE: [u8; 4] = [255, 255, 255, 255];
    let (texture, _) = create_texture_from_rgba(device, queue, 1, 1, &WHITE, Some("flat"));
    let mat = Material::static_mat("flat", texture);
    registry.insert(mat.name.clone(), mat);
}

/// Register the built-in "procedural" material (checkerboard, UV-mapped, trilinear).
#[cfg(feature = "material")]
pub fn register_builtin_procedural(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    registry: &mut MaterialRegistry,
) {
    use crate::backend::create_texture_from_rgba_with_mipmaps;
    const SIZE: u32 = 64;
    let light: [u8; 4] = [255, 255, 255, 255];
    let dark: [u8; 4] = [60, 60, 70, 255];
    let mut rgba = Vec::with_capacity((SIZE as usize) * (SIZE as usize) * 4);
    for y in 0..SIZE {
        for x in 0..SIZE {
            let c = if ((x + y) % 2) == 0 { light } else { dark };
            rgba.extend_from_slice(&c);
        }
    }
    let (texture, _) =
        create_texture_from_rgba_with_mipmaps(device, queue, SIZE, SIZE, &rgba, Some("procedural"));
    let mat = Material::uv_diffuse_mat("procedural", texture);
    registry.insert(mat.name.clone(), mat);
}
