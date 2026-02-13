//! Offscreen framebuffer (FBO) for render-to-texture and readback.

use crate::error::RenderError;

/// Depth format for offscreen framebuffers (matches scene pass).
const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

/// Offscreen render target (FBO): texture, view, and matching depth for 3D scene rendering.
///
/// Use with [`GpuRenderer::render_to_target`](super::gpu::GpuRenderer::render_to_target) to render
/// without presenting. The texture can be used as the source for pixel readback (e.g.
/// [`GpuRenderer::read_pixels_async_from`](super::gpu::GpuRenderer::read_pixels_async_from)).
///
/// FBOs are fixed at creation; they are not auto-resized when the surface or window is resized.
/// Create a new `Framebuffer` and drop the old one when you need a different size.
#[derive(Debug)]
pub struct Framebuffer {
    /// Color attachment (render target and copy source).
    texture: wgpu::Texture,
    /// View of the color texture for render passes.
    view: wgpu::TextureView,
    /// Depth texture for 3D scene (same dimensions as color); kept alive for [`depth_view`](Self::depth_view).
    #[allow(dead_code)]
    depth_texture: wgpu::Texture,
    /// View of the depth texture for render passes.
    depth_view: wgpu::TextureView,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
}

impl Framebuffer {
    /// Returns the color texture (for readback or copy).
    #[must_use]
    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    /// Returns the color view (for render pass color attachment).
    #[must_use]
    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    /// Returns the depth view (for render pass depth attachment).
    #[must_use]
    pub fn depth_view(&self) -> &wgpu::TextureView {
        &self.depth_view
    }

    /// Width in pixels.
    #[must_use]
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Height in pixels.
    #[must_use]
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Color format.
    #[must_use]
    pub fn format(&self) -> wgpu::TextureFormat {
        self.format
    }
}

/// Creates an offscreen framebuffer (color + depth) for the given size and format.
///
/// # Errors
/// Returns error if width or height is zero or if texture creation fails.
pub fn create_framebuffer(
    device: &wgpu::Device,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
) -> Result<Framebuffer, RenderError> {
    if width == 0 || height == 0 {
        return Err(RenderError::WgpuInit(
            "Framebuffer width and height must be greater than zero".to_string(),
        ));
    }

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Framebuffer color"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });

    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Framebuffer depth"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: DEPTH_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });

    let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

    Ok(Framebuffer {
        texture,
        view,
        depth_texture,
        depth_view,
        width,
        height,
        format,
    })
}
