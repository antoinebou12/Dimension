//! Texture creation helpers for materials.

use wgpu::Device;
use wgpu::Queue;

/// Generate RGBA for a procedural checkerboard. Returns (width, height, rgba).
#[must_use]
#[allow(dead_code)]
pub fn procedural_checker_rgba(size: u32, light: [u8; 4], dark: [u8; 4]) -> Vec<u8> {
    let n = (size as usize) * (size as usize) * 4;
    let mut rgba = Vec::with_capacity(n);
    for y in 0..size {
        for x in 0..size {
            let c = if ((x + y) % 2) == 0 { light } else { dark };
            rgba.extend_from_slice(&c);
        }
    }
    rgba
}

/// Compute mip level count for a given width and height.
#[must_use]
fn mip_level_count(width: u32, height: u32) -> u32 {
    let min_dim = width.min(height);
    if min_dim == 0 {
        0
    } else {
        (min_dim.ilog2() + 1).min(16)
    }
}

/// Downsample RGBA by 2x2 box filter. Returns (new_width, new_height, rgba).
/// Input dimensions must be at least 2x2 and even.
#[must_use]
fn downsample_rgba_2x2(width: u32, height: u32, rgba: &[u8]) -> (u32, u32, Vec<u8>) {
    let w = (width / 2).max(1);
    let h = (height / 2).max(1);
    let mut out = Vec::with_capacity((w as usize) * (h as usize) * 4);
    for y in 0..h {
        for x in 0..w {
            let sx = (x * 2).min(width.saturating_sub(1)) as usize;
            let sy = (y * 2).min(height.saturating_sub(1)) as usize;
            let row = (height as usize) * 4;
            let i00 = (sy * row + sx * 4).min(rgba.len().saturating_sub(4));
            let i01 =
                (sy * row + (sx + 1).min(width as usize) * 4).min(rgba.len().saturating_sub(4));
            let i10 =
                ((sy + 1).min(height as usize) * row + sx * 4).min(rgba.len().saturating_sub(4));
            let i11 = ((sy + 1).min(height as usize) * row + (sx + 1).min(width as usize) * 4)
                .min(rgba.len().saturating_sub(4));
            for c in 0..4 {
                let v = (rgba[i00 + c] as u32
                    + rgba[i01 + c] as u32
                    + rgba[i10 + c] as u32
                    + rgba[i11 + c] as u32)
                    / 4;
                out.push(v as u8);
            }
        }
    }
    (w, h, out)
}

/// Create a wgpu texture from raw RGBA bytes (row-major).
///
/// # Panics
/// Panics if `rgba.len() != width * height * 4`.
#[must_use]
pub fn create_texture_from_rgba(
    device: &Device,
    queue: &Queue,
    width: u32,
    height: u32,
    rgba: &[u8],
    label: Option<&str>,
) -> (wgpu::Texture, wgpu::TextureView) {
    create_texture_from_rgba_impl(device, queue, width, height, rgba, label, false)
}

/// Create a wgpu texture from raw RGBA bytes with a full mip chain (trilinear filtering).
///
/// # Panics
/// Panics if `rgba.len() != width * height * 4`.
#[must_use]
pub fn create_texture_from_rgba_with_mipmaps(
    device: &Device,
    queue: &Queue,
    width: u32,
    height: u32,
    rgba: &[u8],
    label: Option<&str>,
) -> (wgpu::Texture, wgpu::TextureView) {
    create_texture_from_rgba_impl(device, queue, width, height, rgba, label, true)
}

fn create_texture_from_rgba_impl(
    device: &Device,
    queue: &Queue,
    width: u32,
    height: u32,
    rgba: &[u8],
    label: Option<&str>,
    with_mipmaps: bool,
) -> (wgpu::Texture, wgpu::TextureView) {
    assert_eq!(rgba.len(), (width as usize) * (height as usize) * 4);
    let mip_count = if with_mipmaps && width > 1 && height > 1 {
        mip_level_count(width, height)
    } else {
        1
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label,
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: mip_count,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let mut w = width;
    let mut h = height;
    let mut data = rgba.to_vec();
    for level in 0..mip_count {
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: level,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(w * 4),
                rows_per_image: Some(h),
            },
            wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
        );
        if level + 1 < mip_count && w > 1 && h > 1 {
            let (nw, nh, next) = downsample_rgba_2x2(w, h, &data);
            w = nw;
            h = nh;
            data = next;
        }
    }
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    (texture, view)
}

#[cfg(feature = "material")]
/// Create a wgpu texture from parsed image data (PNG/JPEG).
///
/// # Errors
/// Returns error if image dimensions are zero or too large.
#[allow(clippy::module_name_repetitions)]
pub fn create_texture_from_image(
    device: &Device,
    queue: &Queue,
    image: &parse::image::ImageData,
    label: Option<&str>,
) -> Result<(wgpu::Texture, wgpu::TextureView), crate::RenderError> {
    let (width, height) = (image.width as u32, image.height as u32);
    if width == 0 || height == 0 {
        return Err(crate::RenderError::MaterialLoad(
            "image has zero width or height".to_string(),
        ));
    }
    let (texture, view) =
        create_texture_from_rgba(device, queue, width, height, &image.data, label);
    Ok((texture, view))
}
