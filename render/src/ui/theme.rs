//! Light/dark theme with semi-transparent colors for UI.

/// RGBA color; alpha < 1 for semi-transparency.
pub type Color = [f32; 4];

/// UI theme: light (white) or dark mode. All colors use alpha for semi-transparency.
#[derive(Clone, Debug)]
pub struct Theme {
    /// Panel/window background.
    pub panel_bg: Color,
    /// Panel border.
    pub panel_border: Color,
    /// Title bar (e.g. window header).
    pub title_bar: Color,
    /// Button default.
    pub button_bg: Color,
    /// Button hover.
    pub button_hover: Color,
    /// Button pressed.
    pub button_pressed: Color,
    /// Slider track.
    pub slider_track: Color,
    /// Slider thumb.
    pub slider_thumb: Color,
    /// Checkbox unchecked.
    pub checkbox_off: Color,
    /// Checkbox checked.
    pub checkbox_on: Color,
    /// Label text color (bitmap glyph text via wgpu_text).
    pub label_text: Color,
    /// Corner radius in pixels for panels and controls (0 = sharp corners).
    pub corner_radius: f32,
}

impl Theme {
    /// Light (white) theme; semi-transparent panels and controls.
    #[must_use]
    pub fn light() -> Self {
        Self {
            panel_bg: [1.0, 1.0, 1.0, 0.92],
            panel_border: [0.85, 0.85, 0.88, 0.95],
            title_bar: [0.95, 0.95, 0.97, 0.95],
            button_bg: [0.97, 0.97, 0.98, 0.9],
            button_hover: [0.90, 0.92, 0.96, 0.95],
            button_pressed: [0.82, 0.85, 0.92, 0.95],
            slider_track: [0.88, 0.88, 0.90, 0.9],
            slider_thumb: [0.35, 0.55, 0.95, 0.95],
            checkbox_off: [0.90, 0.90, 0.92, 0.9],
            checkbox_on: [0.25, 0.55, 0.95, 0.95],
            label_text: [0.0, 0.0, 0.0, 1.0], // Black text on light panels.
            corner_radius: 0.0,
        }
    }

    /// Dark theme; semi-transparent dark panels and controls.
    #[must_use]
    pub fn dark() -> Self {
        Self {
            panel_bg: [0.14, 0.14, 0.16, 0.92],
            panel_border: [0.28, 0.28, 0.32, 0.95],
            title_bar: [0.18, 0.18, 0.20, 0.95],
            button_bg: [0.22, 0.22, 0.26, 0.9],
            button_hover: [0.32, 0.34, 0.40, 0.95],
            button_pressed: [0.40, 0.44, 0.52, 0.95],
            slider_track: [0.24, 0.24, 0.28, 0.9],
            slider_thumb: [0.45, 0.65, 0.98, 0.95],
            checkbox_off: [0.26, 0.26, 0.30, 0.9],
            checkbox_on: [0.45, 0.65, 0.98, 0.95],
            label_text: [0.92, 0.92, 0.95, 1.0],
            corner_radius: 0.0,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::light()
    }
}
