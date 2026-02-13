//! Logical UI controls: window, button, slider, checkbox. Data and state only; no wgpu.

use super::layout::Rect;

/// Interaction state for a control.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ControlState {
    /// Not hovered, not pressed.
    #[default]
    Normal,
    /// Cursor over control.
    Hover,
    /// Mouse down on control.
    Pressed,
}

/// Unique id for a control (used for hit-test and callbacks).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ControlId(pub u32);

/// Button: rect and state; no text in v1.
#[derive(Clone, Debug)]
pub struct Button {
    /// Id for hit-test and callbacks.
    pub id: ControlId,
    /// Bounds in pixel coordinates.
    pub rect: Rect,
    /// Hover / pressed state.
    pub state: ControlState,
}

impl Button {
    /// New button at the given rect.
    #[must_use]
    pub fn new(id: ControlId, rect: Rect) -> Self {
        Self {
            id,
            rect,
            state: ControlState::Normal,
        }
    }
}

/// Default spring stiffness for slider (higher = snappier).
pub const SLIDER_SPRING_STIFFNESS: f32 = 170.0;
/// Default spring damping for slider (0..1, higher = less overshoot).
pub const SLIDER_SPRING_DAMPING: f32 = 0.65;

/// Slider: track rect, value in [0, 1], optional spring animation and drag state.
#[derive(Clone, Debug)]
pub struct Slider {
    /// Id for hit-test.
    pub id: ControlId,
    /// Track bounds (full range).
    pub rect: Rect,
    /// Current value in [0, 1] (displayed; animates toward [`Self::target_value`] when using spring).
    pub value: f32,
    /// Target value for spring animation; [`Self::value`] moves toward this each frame when [`Self::update_spring`] is called.
    pub target_value: f32,
    /// Spring velocity (internal).
    pub velocity: f32,
    /// Hover / dragging state.
    pub state: ControlState,
    /// True while user is dragging the thumb.
    pub dragging: bool,
}

impl Slider {
    /// New slider at the given track rect; value in [0, 1].
    #[must_use]
    pub fn new(id: ControlId, rect: Rect, value: f32) -> Self {
        let v = value.clamp(0.0, 1.0);
        Self {
            id,
            rect,
            value: v,
            target_value: v,
            velocity: 0.0,
            state: ControlState::Normal,
            dragging: false,
        }
    }

    /// Set target value; [`Self::value`] will animate toward it when [`crate::UiLayer::update_springs`] is called.
    pub fn set_target_value(&mut self, target: f32) {
        self.target_value = target.clamp(0.0, 1.0);
    }

    /// Advance spring physics by `dt` seconds. Call each frame for smooth animation.
    pub fn update_spring(&mut self, dt: f32) {
        if self.dragging {
            return;
        }
        let stiffness = SLIDER_SPRING_STIFFNESS;
        let damping = SLIDER_SPRING_DAMPING;
        self.velocity += (self.target_value - self.value) * stiffness * dt;
        self.velocity *= 1.0 - (1.0 - damping).powf(dt * 60.0);
        self.value += self.velocity * dt;
        self.value = self.value.clamp(0.0, 1.0);
        if (self.value - self.target_value).abs() < 1e-4 && self.velocity.abs() < 1e-4 {
            self.value = self.target_value;
            self.velocity = 0.0;
        }
    }

    /// Thumb size: fixed height = track height, width = 12px or 1/8 of track width (min 8).
    #[must_use]
    pub fn thumb_rect(&self, viewport_width: f32) -> Rect {
        let tw = self.rect.w.min(viewport_width / 8.0).max(8.0);
        let th = self.rect.h;
        let t = self.value.clamp(0.0, 1.0);
        let x = self.rect.x + t * (self.rect.w - tw);
        Rect::new(x, self.rect.y, tw, th)
    }
}

/// Checkbox: rect and checked state.
#[derive(Clone, Debug)]
pub struct Checkbox {
    /// Id for hit-test and toggle.
    pub id: ControlId,
    /// Bounds in pixel coordinates.
    pub rect: Rect,
    /// Checked state.
    pub checked: bool,
    /// Hover / pressed state.
    pub state: ControlState,
}

impl Checkbox {
    /// New checkbox at the given rect.
    #[must_use]
    pub fn new(id: ControlId, rect: Rect, checked: bool) -> Self {
        Self {
            id,
            rect,
            checked,
            state: ControlState::Normal,
        }
    }
}

/// Horizontal alignment of label text within its rect.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LabelTextAlign {
    /// Text aligned to the left of the label rect.
    #[default]
    Left,
    /// Text centered horizontally within the label rect.
    Center,
    /// Text aligned to the right of the label rect.
    Right,
}

/// Label: non-interactive text line (rect + string).
#[derive(Clone, Debug)]
pub struct Label {
    /// Id for layout ordering.
    pub id: ControlId,
    /// Bounds in pixel coordinates.
    pub rect: Rect,
    /// Display text (e.g. "FPS: 60").
    pub text: String,
    /// If false, only text is drawn (no background quad). Use for labels on buttons.
    pub draw_background: bool,
    /// Horizontal alignment of text within the label rect.
    pub alignment: LabelTextAlign,
}

impl Label {
    /// New label at the given rect with the given text. Draws a background quad behind the text.
    #[must_use]
    pub fn new(id: ControlId, rect: Rect, text: String) -> Self {
        Self {
            id,
            rect,
            text,
            draw_background: true,
            alignment: LabelTextAlign::Left,
        }
    }

    /// New label that draws only text (no background). Use for button labels; the button provides the background.
    #[must_use]
    pub fn new_overlay(id: ControlId, rect: Rect, text: String) -> Self {
        Self {
            id,
            rect,
            text,
            draw_background: false,
            alignment: LabelTextAlign::Left,
        }
    }
}

/// Child control in a window: button, slider, checkbox, or label.
#[derive(Clone, Debug)]
pub enum WindowChild {
    /// Button control.
    Button(Button),
    /// Slider control.
    Slider(Slider),
    /// Checkbox control.
    Checkbox(Checkbox),
    /// Label (non-interactive text line).
    Label(Label),
}

/// Window: panel rect, optional title bar height, and children.
/// Optional scroll: set [`Self::content_height`] larger than body height to enable scrolling.
#[derive(Clone, Debug)]
pub struct Window {
    /// Id for hit-test ordering.
    pub id: ControlId,
    /// Full window bounds (panel).
    pub rect: Rect,
    /// Title bar height in pixels; 0 means no title bar.
    pub title_bar_height: f32,
    /// Total height of content (for scrollable windows). 0 = not scrollable.
    pub content_height: f32,
    /// Current scroll offset (0 .. max(0, content_height - body.h)). Used when [`Self::content_height`] > body height.
    pub scroll_y: f32,
    /// Child controls (buttons, sliders, checkboxes).
    pub children: Vec<WindowChild>,
}

impl Window {
    /// New window with the given rect and no children.
    #[must_use]
    pub fn new(id: ControlId, rect: Rect) -> Self {
        Self {
            id,
            rect,
            title_bar_height: 24.0,
            content_height: 0.0,
            scroll_y: 0.0,
            children: Vec::new(),
        }
    }

    /// True if this window has scrollable content (content height exceeds body height).
    #[must_use]
    pub fn is_scrollable(&self) -> bool {
        let body = self.body_rect();
        self.content_height > body.h
    }

    /// Maximum scroll offset (clamp [`Self::scroll_y`] to 0..= this).
    #[must_use]
    pub fn max_scroll_y(&self) -> f32 {
        let body = self.body_rect();
        (self.content_height - body.h).max(0.0)
    }

    /// Add a button to this window.
    pub fn add_button(&mut self, button: Button) {
        self.children.push(WindowChild::Button(button));
    }

    /// Add a slider to this window.
    pub fn add_slider(&mut self, slider: Slider) {
        self.children.push(WindowChild::Slider(slider));
    }

    /// Add a checkbox to this window.
    pub fn add_checkbox(&mut self, checkbox: Checkbox) {
        self.children.push(WindowChild::Checkbox(checkbox));
    }

    /// Add a label to this window.
    pub fn add_label(&mut self, label: Label) {
        self.children.push(WindowChild::Label(label));
    }

    /// Remove all child controls. Use when repopulating the window (e.g. after switching content).
    pub fn clear_children(&mut self) {
        self.children.clear();
    }

    /// Body rect (below title bar).
    #[must_use]
    pub fn body_rect(&self) -> Rect {
        if self.title_bar_height <= 0.0 {
            return self.rect;
        }
        Rect::new(
            self.rect.x,
            self.rect.y + self.title_bar_height,
            self.rect.w,
            self.rect.h - self.title_bar_height,
        )
    }
}
