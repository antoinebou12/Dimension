//! PNG and JPEG parsers using minipng and jpeg-decoder.
//!
//! Returns raw RGBA u8 pixels (row-major).

use crate::error::ParseError;
use mathlib::Cube;
use std::io::Cursor;

/// Decoded image: width, height, RGBA pixels (row-major).
#[derive(Clone, Debug)]
pub struct ImageData {
    /// Width in pixels.
    pub width: usize,
    /// Height in pixels.
    pub height: usize,
    /// RGBA pixels, row-major, 4 bytes per pixel.
    pub data: Vec<u8>,
}

impl ImageData {
    /// Converts to mathlib Cube (height, width, 4) for channel-wise ops.
    pub fn to_cube(&self) -> Cube<f32> {
        let mut cube = Cube::with_dimensions(self.height, self.width, 4);
        let data = cube.data_mut();
        for (i, &b) in self.data.iter().enumerate() {
            data[i] = b as f32 / 255.0;
        }
        cube
    }
}

/// Parses PNG from bytes.
///
/// Returns RGBA 8bpc. Non-RGBA images are converted to RGBA.
///
/// # Errors
///
/// Returns [`ParseError`](crate::ParseError) on invalid PNG.
pub fn parse_png(data: &[u8]) -> Result<ImageData, ParseError> {
    let header = minipng::decode_png_header(data).map_err(|e| ParseError::Io(e.to_string()))?;
    let required = header.required_bytes_rgba8bpc();
    let mut buffer = vec![0u8; required];
    let mut image =
        minipng::decode_png(data, &mut buffer).map_err(|e| ParseError::Io(e.to_string()))?;
    image
        .convert_to_rgba8bpc()
        .map_err(|e| ParseError::Io(e.to_string()))?;
    Ok(ImageData {
        width: image.width() as usize,
        height: image.height() as usize,
        data: image.pixels().to_vec(),
    })
}

/// Parses JPEG from bytes.
///
/// Returns RGBA 8bpc. Grayscale/YCbCr are converted to RGBA.
///
/// # Errors
///
/// Returns [`ParseError`](crate::ParseError) on invalid JPEG.
pub fn parse_jpeg(data: &[u8]) -> Result<ImageData, ParseError> {
    let mut cursor = Cursor::new(data);
    let mut decoder = jpeg_decoder::Decoder::new(&mut cursor);
    let pixels = decoder
        .decode()
        .map_err(|e| ParseError::Io(e.to_string()))?;
    let info = decoder
        .info()
        .ok_or_else(|| ParseError::Io("no JPEG info".to_string()))?;

    let (width, height) = (info.width as usize, info.height as usize);
    let data = match info.pixel_format {
        jpeg_decoder::PixelFormat::RGB24 => rgb_to_rgba(&pixels),
        jpeg_decoder::PixelFormat::L8 => l8_to_rgba(&pixels),
        jpeg_decoder::PixelFormat::CMYK32 => cmyk_to_rgba(&pixels),
        _ => rgb_to_rgba(&pixels),
    };

    Ok(ImageData {
        width,
        height,
        data,
    })
}

fn rgb_to_rgba(rgb: &[u8]) -> Vec<u8> {
    let mut rgba = Vec::with_capacity(rgb.len() / 3 * 4);
    for chunk in rgb.chunks_exact(3) {
        rgba.extend_from_slice(chunk);
        rgba.push(255);
    }
    rgba
}

fn l8_to_rgba(l: &[u8]) -> Vec<u8> {
    l.iter().flat_map(|&v| [v, v, v, 255]).collect()
}

fn cmyk_to_rgba(cmyk: &[u8]) -> Vec<u8> {
    let mut rgba = Vec::with_capacity(cmyk.len() / 4 * 4);
    for chunk in cmyk.chunks_exact(4) {
        let c = chunk[0] as f32 / 255.0;
        let m = chunk[1] as f32 / 255.0;
        let y = chunk[2] as f32 / 255.0;
        let k = chunk[3] as f32 / 255.0;
        let r = (1.0 - c) * (1.0 - k);
        let g = (1.0 - m) * (1.0 - k);
        let b = (1.0 - y) * (1.0 - k);
        rgba.push((r * 255.0) as u8);
        rgba.push((g * 255.0) as u8);
        rgba.push((b * 255.0) as u8);
        rgba.push(255);
    }
    rgba
}
