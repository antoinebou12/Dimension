//! wgpu-rendered UI layer: semi-transparent window, button, slider, checkbox; light/dark theme.

mod components;
mod layout;
mod mesh;
mod render_pass;
mod theme;

pub use components::{
    Button, Checkbox, ControlId, ControlState, Label, LabelTextAlign, Slider, Window, WindowChild,
    SLIDER_SPRING_DAMPING, SLIDER_SPRING_STIFFNESS,
};
pub use layout::{vertical_stack, Rect, VerticalLayout};
pub use render_pass::UiRenderPass;
pub use theme::Theme;

use components::WindowChild as WC;

/// UI layer: windows, theme, hit-test state, and render pass. Integrates with [`crate::Engine`].
pub struct UiLayer {
    /// Current theme (light or dark).
    pub theme: Theme,
    /// Root windows (last = front for hit-test).
    pub windows: Vec<Window>,
    /// Viewport size for ortho and layout.
    viewport_width: f32,
    viewport_height: f32,
    /// Render pass (pipeline + buffers).
    pass: UiRenderPass,
    /// Cursor position (pixel).
    cursor: (f32, f32),
    /// Control under cursor.
    hover: Option<HoverTarget>,
    /// Control that has mouse pressed (for click/drag).
    pressed: Option<HoverTarget>,
    /// Last button clicked (set on mouse up when released over same button). Consumed by [`Self::take_clicked_control`].
    last_clicked: Option<ControlId>,
}

/// Identifies which control is hovered or pressed (window index + child type/index).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct HoverTarget {
    window_idx: usize,
    child_idx: usize,
    is_slider_thumb: bool,
}

impl UiLayer {
    /// Create a new UI layer. Call [`Self::resize`] when viewport changes.
    ///
    /// # Errors
    /// Returns error if pipeline creation fails.
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        width: f32,
        height: f32,
    ) -> Result<Self, crate::error::RenderError> {
        let pass = UiRenderPass::new(device, queue, format, width, height, None)?;
        Ok(Self {
            theme: Theme::default(),
            windows: Vec::new(),
            viewport_width: width,
            viewport_height: height,
            pass,
            cursor: (0.0, 0.0),
            hover: None,
            pressed: None,
            last_clicked: None,
        })
    }

    /// Set the theme (light or dark).
    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }

    /// Resize the UI ortho and viewport; call when the window/canvas is resized.
    /// `width` and `height` must match the render target (e.g. surface config) so that
    /// text and layout render in the correct place.
    pub fn resize(&mut self, queue: &wgpu::Queue, width: f32, height: f32) {
        self.viewport_width = width;
        self.viewport_height = height;
        self.pass.resize(queue, width, height);
    }

    /// Add a window to the layer (pushed on top for hit-test).
    pub fn add_window(&mut self, window: Window) {
        self.windows.push(window);
    }

    /// Mutable reference to the theme.
    pub fn theme_mut(&mut self) -> &mut Theme {
        &mut self.theme
    }

    /// Mutable reference to the windows list.
    pub fn windows_mut(&mut self) -> &mut Vec<Window> {
        &mut self.windows
    }

    /// Run hit-test and update hover state. Call on cursor move.
    pub fn ui_mouse_move(&mut self, x: f32, y: f32) {
        self.cursor = (x, y);
        self.hover = self.hit_test(x, y);

        if let Some(ref pressed) = self.pressed {
            if let Some(ref hover) = self.hover {
                if pressed.window_idx == hover.window_idx
                    && pressed.child_idx == hover.child_idx
                    && (pressed.is_slider_thumb == hover.is_slider_thumb)
                {
                    if pressed.is_slider_thumb {
                        if let Some((wi, ci)) =
                            self.get_slider_mut(pressed.window_idx, pressed.child_idx)
                        {
                            let s = &mut self.windows[wi].children[ci];
                            if let WC::Slider(slider) = s {
                                let tr = slider.thumb_rect(self.viewport_width);
                                let t = if tr.w > 0.0 {
                                    ((x - slider.rect.x - tr.w * 0.5) / (slider.rect.w - tr.w))
                                        .clamp(0.0, 1.0)
                                } else {
                                    slider.value
                                };
                                slider.value = t;
                                slider.target_value = t;
                            }
                        }
                    }
                    self.sync_control_states();
                    return;
                }
            }
        }
        self.sync_control_states();
    }

    /// Record mouse down; start hover as pressed. Call when left button is pressed.
    pub fn ui_mouse_down(&mut self) {
        self.pressed = self.hover;
        if let Some(ref t) = self.pressed {
            if t.is_slider_thumb {
                if let Some((wi, ci)) = self.get_slider_mut(t.window_idx, t.child_idx) {
                    if let WC::Slider(s) = &mut self.windows[wi].children[ci] {
                        s.dragging = true;
                        s.state = ControlState::Pressed;
                    }
                }
            } else {
                self.set_child_state(t.window_idx, t.child_idx, ControlState::Pressed, false);
            }
        }
    }

    /// Record mouse up; clear pressed, emit click/toggle for button/checkbox, stop slider drag.
    pub fn ui_mouse_up(&mut self) {
        let was_pressed = self.pressed.take();
        if let Some(t) = was_pressed {
            if t.is_slider_thumb {
                if let Some((wi, ci)) = self.get_slider_mut(t.window_idx, t.child_idx) {
                    if let WC::Slider(s) = &mut self.windows[wi].children[ci] {
                        s.dragging = false;
                        s.state = if self.hover.as_ref() == Some(&t) {
                            ControlState::Hover
                        } else {
                            ControlState::Normal
                        };
                    }
                }
            } else {
                if let Some((wi, ci)) = self.get_child(t.window_idx, t.child_idx) {
                    match &self.windows[wi].children[ci] {
                        WC::Button(b) => {
                            if self.hover.as_ref() == Some(&t) {
                                self.last_clicked = Some(b.id);
                            }
                        }
                        WC::Checkbox(_) => {
                            if self.hover.as_ref() == Some(&t) {
                                if let WC::Checkbox(cb) = &mut self.windows[wi].children[ci] {
                                    cb.checked = !cb.checked;
                                }
                            }
                        }
                        WC::Slider(_) => {}
                        WC::Label(_) => {}
                    }
                }
                self.set_child_state(t.window_idx, t.child_idx, ControlState::Normal, true);
            }
        }
        self.sync_control_states();
    }

    /// Returns and clears the last clicked button's [`ControlId`], if any. Call after [`Self::ui_mouse_up`] to react to button clicks.
    #[must_use]
    pub fn take_clicked_control(&mut self) -> Option<ControlId> {
        self.last_clicked.take()
    }

    /// Scroll the topmost scrollable window under the current cursor by `delta` (positive = content up).
    /// Call from platform on mouse wheel when cursor is over UI.
    pub fn scroll_window_at_cursor(&mut self, delta: f32) {
        let (x, y) = self.cursor;
        for w in self.windows.iter_mut().rev() {
            if !w.rect.contains(x, y) {
                continue;
            }
            let body = w.body_rect();
            if !body.contains(x, y) {
                continue;
            }
            if w.is_scrollable() {
                let max_scroll = w.max_scroll_y();
                w.scroll_y = (w.scroll_y + delta).clamp(0.0, max_scroll);
                return;
            }
        }
    }

    /// Returns true if the cursor is over any UI control or inside any window (e.g. so platform can skip orbit).
    #[must_use]
    pub fn is_cursor_over_ui(&self) -> bool {
        self.hover.is_some() || self.cursor_in_any_window()
    }

    /// True if the cursor is inside any window's rect (including label-only areas like the stats panel).
    fn cursor_in_any_window(&self) -> bool {
        let (x, y) = self.cursor;
        self.windows.iter().any(|w| w.rect.contains(x, y))
    }

    /// Advance spring animation for all sliders by `dt` seconds. Call each frame before [`Self::prepare_render`].
    pub fn update_springs(&mut self, dt: f32) {
        for w in &mut self.windows {
            for child in &mut w.children {
                if let WC::Slider(s) = child {
                    s.update_spring(dt);
                }
            }
        }
    }

    /// Upload UI mesh for this frame. Call before encoding the UI pass into the same encoder as the scene pass.
    pub fn prepare_render(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.pass.update_mesh(
            device,
            queue,
            &self.windows,
            &self.theme,
            self.viewport_width,
        );
    }

    /// Encode UI draw into an existing render pass (use `LoadOp::Load` so the scene is visible underneath).
    pub fn encode_into_pass<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        self.pass.draw(render_pass);
    }

    fn hit_test(&self, x: f32, y: f32) -> Option<HoverTarget> {
        for (wi, w) in self.windows.iter().enumerate().rev() {
            if !w.rect.contains(x, y) {
                continue;
            }
            let scroll_y = if w.is_scrollable() { w.scroll_y } else { 0.0 };
            for (ci, child) in w.children.iter().enumerate().rev() {
                let rect = self.child_screen_rect(child, scroll_y);
                match child {
                    WC::Slider(s) => {
                        let thumb = s.thumb_rect(self.viewport_width);
                        let thumb_screen = Rect::new(thumb.x, thumb.y - scroll_y, thumb.w, thumb.h);
                        if thumb_screen.contains(x, y) {
                            return Some(HoverTarget {
                                window_idx: wi,
                                child_idx: ci,
                                is_slider_thumb: true,
                            });
                        }
                        if rect.contains(x, y) {
                            return Some(HoverTarget {
                                window_idx: wi,
                                child_idx: ci,
                                is_slider_thumb: false,
                            });
                        }
                    }
                    WC::Button(_) if rect.contains(x, y) => {
                        return Some(HoverTarget {
                            window_idx: wi,
                            child_idx: ci,
                            is_slider_thumb: false,
                        });
                    }
                    WC::Checkbox(_) if rect.contains(x, y) => {
                        return Some(HoverTarget {
                            window_idx: wi,
                            child_idx: ci,
                            is_slider_thumb: false,
                        });
                    }
                    _ => {}
                }
            }
            // Inside this window but no interactive control; do not hit windows below.
            return None;
        }
        None
    }

    fn child_screen_rect(&self, child: &WC, scroll_y: f32) -> Rect {
        let r = match child {
            WC::Button(b) => b.rect,
            WC::Slider(s) => s.rect,
            WC::Checkbox(c) => c.rect,
            WC::Label(l) => l.rect,
        };
        if scroll_y > 0.0 {
            Rect::new(r.x, r.y - scroll_y, r.w, r.h)
        } else {
            r
        }
    }

    fn get_child(&self, wi: usize, ci: usize) -> Option<(usize, usize)> {
        if wi < self.windows.len() && ci < self.windows[wi].children.len() {
            Some((wi, ci))
        } else {
            None
        }
    }

    fn get_slider_mut(&mut self, wi: usize, ci: usize) -> Option<(usize, usize)> {
        if wi >= self.windows.len() || ci >= self.windows[wi].children.len() {
            return None;
        }
        if matches!(self.windows[wi].children[ci], WC::Slider(_)) {
            Some((wi, ci))
        } else {
            None
        }
    }

    fn set_child_state(&mut self, wi: usize, ci: usize, state: ControlState, clear_pressed: bool) {
        if wi >= self.windows.len() || ci >= self.windows[wi].children.len() {
            return;
        }
        let child = &mut self.windows[wi].children[ci];
        match child {
            WC::Button(b) => {
                b.state = state;
            }
            WC::Checkbox(c) => {
                c.state = state;
            }
            WC::Slider(s) => {
                if !s.dragging || clear_pressed {
                    s.state = state;
                }
            }
            WC::Label(_) => {}
        }
    }

    fn sync_control_states(&mut self) {
        for (wi, w) in self.windows.iter_mut().enumerate() {
            for (ci, child) in w.children.iter_mut().enumerate() {
                let is_hovered = self
                    .hover
                    .as_ref()
                    .map(|h| h.window_idx == wi && h.child_idx == ci)
                    .unwrap_or(false);
                let is_pressed = self
                    .pressed
                    .as_ref()
                    .map(|p| p.window_idx == wi && p.child_idx == ci)
                    .unwrap_or(false);
                let state = if is_pressed {
                    ControlState::Pressed
                } else if is_hovered {
                    ControlState::Hover
                } else {
                    ControlState::Normal
                };
                match child {
                    WC::Button(b) => b.state = state,
                    WC::Checkbox(c) => c.state = state,
                    WC::Slider(s) => {
                        if !s.dragging {
                            s.state = state;
                        }
                    }
                    WC::Label(_) => {}
                }
            }
        }
    }
}
