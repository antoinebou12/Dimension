//! Conversions between color spaces: RGBA, RGB, HSV, hex.

use super::types::{Hsv, Rgb, Rgba};

/// RGBA to RGB: drops alpha (no premul). Use when alpha is opaque or irrelevant.
#[inline]
pub fn rgba_to_rgb(rgba: Rgba) -> Rgb {
    [rgba[0], rgba[1], rgba[2]]
}

/// RGB to RGBA: adds alpha channel.
#[inline]
pub fn rgb_to_rgba(rgb: Rgb, a: u8) -> Rgba {
    [rgb[0], rgb[1], rgb[2], a]
}

/// RGB (0–255) to HSV. H in [0, 360), S and V in [0, 1].
#[allow(clippy::many_single_char_names)]
pub fn rgb_to_hsv(r: u8, g: u8, b: u8) -> Hsv {
    let rn = f64::from(r) / 255.0;
    let gn = f64::from(g) / 255.0;
    let bn = f64::from(b) / 255.0;
    let max = rn.max(gn).max(bn);
    let min = rn.min(gn).min(bn);
    let c = max - min;
    let v = max;
    let s = if max > 0.0 { c / max } else { 0.0 };
    let h = if c <= 0.0 {
        0.0
    } else if (max - rn).abs() < 1e-10 {
        60.0 * (((gn - bn) / c) % 6.0)
    } else if (max - gn).abs() < 1e-10 {
        60.0 * ((bn - rn) / c + 2.0)
    } else {
        60.0 * ((rn - gn) / c + 4.0)
    };
    let h = if h < 0.0 { h + 360.0 } else { h };
    Hsv {
        h: h.clamp(0.0, 360.0),
        s: s.clamp(0.0, 1.0),
        v: v.clamp(0.0, 1.0),
    }
}

/// HSV to RGB (0–255). H in [0, 360), S and V in [0, 1].
#[allow(
    clippy::many_single_char_names,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
pub fn hsv_to_rgb(h: f64, s: f64, v: f64) -> Rgb {
    let h = h.clamp(0.0, 360.0);
    let s = s.clamp(0.0, 1.0);
    let v = v.clamp(0.0, 1.0);
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    let (r, g, b) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    [
        ((r + m) * 255.0).round().clamp(0.0, 255.0) as u8,
        ((g + m) * 255.0).round().clamp(0.0, 255.0) as u8,
        ((b + m) * 255.0).round().clamp(0.0, 255.0) as u8,
    ]
}

/// RGB to hex string, e.g. `#RRGGBB`.
#[inline]
pub fn rgb_to_hex(rgb: Rgb) -> String {
    format!("#{:02x}{:02x}{:02x}", rgb[0], rgb[1], rgb[2])
}

/// Parse hex string to RGB. Accepts `#RRGGBB` or `RRGGBB`. Returns `None` if invalid.
pub fn hex_to_rgb(hex: &str) -> Option<Rgb> {
    let s = hex.strip_prefix('#').unwrap_or(hex);
    if s.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&s[0..2], 16).ok()?;
    let g = u8::from_str_radix(&s[2..4], 16).ok()?;
    let b = u8::from_str_radix(&s[4..6], 16).ok()?;
    Some([r, g, b])
}

/// RGBA to hex string, e.g. `#RRGGBBAA`.
#[inline]
pub fn rgba_to_hex(rgba: Rgba) -> String {
    format!(
        "#{:02x}{:02x}{:02x}{:02x}",
        rgba[0], rgba[1], rgba[2], rgba[3]
    )
}

/// Parse hex string to RGBA. Accepts `#RRGGBBAA` or `RRGGBBAA` (8 chars), or `#RRGGBB`/`RRGGBB` (alpha 255). Returns `None` if invalid.
pub fn hex_to_rgba(hex: &str) -> Option<Rgba> {
    let s = hex.strip_prefix('#').unwrap_or(hex);
    if s.len() == 6 {
        let [r, g, b] = hex_to_rgb(hex)?;
        return Some([r, g, b, 255]);
    }
    if s.len() != 8 {
        return None;
    }
    let r = u8::from_str_radix(&s[0..2], 16).ok()?;
    let g = u8::from_str_radix(&s[2..4], 16).ok()?;
    let b = u8::from_str_radix(&s[4..6], 16).ok()?;
    let a = u8::from_str_radix(&s[6..8], 16).ok()?;
    Some([r, g, b, a])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rgba_to_rgb_drops_alpha() {
        assert_eq!(rgba_to_rgb([1, 2, 3, 255]), [1, 2, 3]);
    }

    #[test]
    fn rgb_to_rgba_adds_alpha() {
        assert_eq!(rgb_to_rgba([1, 2, 3], 255), [1, 2, 3, 255]);
    }

    #[test]
    fn rgb_hsv_roundtrip() {
        let rgb = [100, 150, 200];
        let hsv = rgb_to_hsv(rgb[0], rgb[1], rgb[2]);
        let back = hsv_to_rgb(hsv.h, hsv.s, hsv.v);
        assert_eq!(back, rgb);
    }

    #[test]
    fn black_white_hsv() {
        let hsv_black = rgb_to_hsv(0, 0, 0);
        assert!(hsv_black.v < 1e-6);
        let hsv_white = rgb_to_hsv(255, 255, 255);
        assert!((hsv_white.v - 1.0).abs() < 1e-6);
        assert!(hsv_white.s < 1e-6);
    }

    #[test]
    fn rgb_hex_roundtrip() {
        let rgb = [10, 20, 30];
        let hex = rgb_to_hex(rgb);
        assert_eq!(hex_to_rgb(&hex), Some(rgb));
    }

    #[test]
    fn hex_without_hash() {
        assert_eq!(hex_to_rgb("0a141e"), Some([10, 20, 30]));
    }

    #[test]
    fn rgba_hex_roundtrip() {
        let rgba = [10, 20, 30, 128];
        let hex = rgba_to_hex(rgba);
        assert_eq!(hex_to_rgba(&hex), Some(rgba));
    }
}
