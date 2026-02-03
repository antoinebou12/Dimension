//! Color types and conversions for heightmaps and procedural color (RGBA, RGB, HSV, hex).
//!
//! - **types**: [`Rgb`], [`Rgba`], [`Hsv`].
//! - **convert**: [`rgba_to_rgb`], [`rgb_to_rgba`], [`rgb_to_hsv`], [`hsv_to_rgb`], [`rgb_to_hex`], [`hex_to_rgb`], [`rgba_to_hex`], [`hex_to_rgba`].
//! - **palette**: [`height_to_rgb`], [`height_to_rgba`] (elevation-like gradient).

pub mod convert;
pub mod palette;
pub mod types;

pub use convert::{
    hex_to_rgb, hex_to_rgba, hsv_to_rgb, rgb_to_hex, rgb_to_hsv, rgb_to_rgba, rgba_to_hex,
    rgba_to_rgb,
};
pub use palette::{height_to_rgb, height_to_rgba};
pub use types::{Hsv, Rgb, Rgba};
