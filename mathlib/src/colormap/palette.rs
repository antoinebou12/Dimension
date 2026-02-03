//! Height-to-color palettes for heightmaps (e.g. elevation: blue → cyan/green → yellow/red).

use super::types::{Rgb, Rgba};

/// Elevation-like palette: blue (low) → cyan/green (mid) → yellow/red (high).
/// `h` must be in [0, 1]; result is clamped.
#[inline]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn height_to_rgb(h: f64) -> Rgb {
    let h = h.clamp(0.0, 1.0);
    let r = if h < 0.5 {
        0
    } else {
        (255.0 * 2.0 * (h - 0.5)).round() as u8
    };
    let g = if h < 0.5 {
        (255.0 * 2.0 * h).round() as u8
    } else {
        255
    };
    let b = if h < 0.5 {
        255
    } else {
        (255.0 * 2.0 * (1.0 - h)).round() as u8
    };
    [r, g, b]
}

/// Same as [`height_to_rgb`] with alpha = 255.
#[inline]
pub fn height_to_rgba(h: f64) -> Rgba {
    let [r, g, b] = height_to_rgb(h);
    [r, g, b, 255]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn height_to_rgb_zero() {
        let rgb = height_to_rgb(0.0);
        assert_eq!(rgb, [0, 0, 255]);
    }

    #[test]
    fn height_to_rgb_one() {
        let rgb = height_to_rgb(1.0);
        assert_eq!(rgb, [255, 255, 0]);
    }

    #[test]
    fn height_to_rgb_mid() {
        let rgb = height_to_rgb(0.5);
        assert_eq!(rgb[1], 255);
    }

    #[test]
    fn height_to_rgba_has_alpha_255() {
        assert_eq!(height_to_rgba(0.5)[3], 255);
    }
}
