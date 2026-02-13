//! WASM bindings for colormap (elevation-style palette).

use wasm_bindgen::prelude::*;

/// Elevation-style colormap: blue (h=0) → green (h=0.5) → yellow/red (h=1).
/// `h` is clamped to [0, 1]. Returns [r, g, b] in 0..255.
#[wasm_bindgen(js_name = heightToRgb)]
pub fn height_to_rgb_wasm(h: f64) -> Vec<u8> {
    crate::colormap::height_to_rgb(h).to_vec()
}
