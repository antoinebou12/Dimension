//! Color types for the colormap module.

/// RGB color: red, green, blue in [0, 255].
pub type Rgb = [u8; 3];

/// RGBA color: red, green, blue, alpha in [0, 255].
pub type Rgba = [u8; 4];

/// HSV color: hue in [0, 360), saturation and value in [0, 1].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hsv {
    /// Hue in degrees [0, 360).
    pub h: f64,
    /// Saturation [0, 1].
    pub s: f64,
    /// Value (brightness) [0, 1].
    pub v: f64,
}
