//! Build vertex and index buffers for the UI tree from theme and component state.
//! Produces solid quads only (optionally with rounded corners via theme corner_radius); text is rendered separately via wgpu_text with bounds and alignment from [`Label`](crate::ui::Label).

use crate::backend::Vertex;
use crate::ui::components::{ControlState, Window, WindowChild};
use crate::ui::layout::Rect;
use crate::ui::theme::Theme;

/// Sentinel UV for solid-color quads (shader skips texture sampling).
const SENTINEL_UV: f32 = 999.0;

const BORDER_WIDTH: f32 = 1.0;

/// Append a 1px border around the outside of a rect (no rounding).
fn push_border(vertices: &mut Vec<Vertex>, indices: &mut Vec<u16>, rect: Rect, color: [f32; 4]) {
    let x0 = rect.x;
    let y0 = rect.y;
    let x1 = rect.right();
    let y1 = rect.bottom();
    let b = BORDER_WIDTH;
    push_quad(vertices, indices, Rect::new(x0, y0, rect.w, b), color, 0.0); // top
    push_quad(
        vertices,
        indices,
        Rect::new(x0, y1 - b, rect.w, b),
        color,
        0.0,
    ); // bottom
    push_quad(vertices, indices, Rect::new(x0, y0, b, rect.h), color, 0.0); // left
    push_quad(
        vertices,
        indices,
        Rect::new(x1 - b, y0, b, rect.h),
        color,
        0.0,
    ); // right
}

/// Append a solid-color quad (two triangles). Uses sentinel UV so the shader draws vertex color only.
/// When corner_radius > 0, the fragment shader draws a rounded rect.
fn push_quad(
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u16>,
    rect: Rect,
    color: [f32; 4],
    corner_radius: f32,
) {
    let base = vertices.len() as u16;
    let x0 = rect.x;
    let y0 = rect.y;
    let x1 = rect.right();
    let y1 = rect.bottom();
    let uv = [SENTINEL_UV, SENTINEL_UV];
    let rmin = [x0, y0];
    let rmax = [x1, y1];
    vertices.push(Vertex {
        position: [x0, y0, 0.0],
        uv,
        color,
        rect_min: rmin,
        rect_max: rmax,
        corner_radius,
    });
    vertices.push(Vertex {
        position: [x1, y0, 0.0],
        uv,
        color,
        rect_min: rmin,
        rect_max: rmax,
        corner_radius,
    });
    vertices.push(Vertex {
        position: [x1, y1, 0.0],
        uv,
        color,
        rect_min: rmin,
        rect_max: rmax,
        corner_radius,
    });
    vertices.push(Vertex {
        position: [x0, y1, 0.0],
        uv,
        color,
        rect_min: rmin,
        rect_max: rmax,
        corner_radius,
    });
    indices.extend([base, base + 1, base + 2, base, base + 2, base + 3]);
}

/// Per-window draw range for scissor: body rect and index range into the combined index buffer.
#[derive(Clone, Debug)]
pub struct WindowDrawRange {
    /// Body rect (viewport for scissor when window is scrollable).
    pub body_rect: Rect,
    /// Start index in the index buffer for this window's quads.
    pub index_start: u32,
    /// Number of indices for this window.
    pub index_count: u32,
}

/// Build vertices and indices for the given list of windows and theme.
/// For scrollable windows, child rects are offset by -scroll_y so content scrolls; use [`WindowDrawRange`] with scissor when drawing.
#[must_use]
pub fn build_ui_mesh(
    windows: &[Window],
    theme: &Theme,
    viewport_width: f32,
) -> (Vec<Vertex>, Vec<u16>, Vec<WindowDrawRange>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut ranges = Vec::with_capacity(windows.len());

    for window in windows {
        let body = window.body_rect();
        let scroll_y = if window.is_scrollable() {
            window.scroll_y.clamp(0.0, window.max_scroll_y())
        } else {
            0.0
        };
        let index_start = indices.len() as u32;

        push_quad(
            &mut vertices,
            &mut indices,
            window.rect,
            theme.panel_bg,
            theme.corner_radius,
        );
        push_border(&mut vertices, &mut indices, window.rect, theme.panel_border);
        if window.title_bar_height > 0.0 {
            let title_rect = Rect::new(
                window.rect.x,
                window.rect.y,
                window.rect.w,
                window.title_bar_height,
            );
            push_quad(
                &mut vertices,
                &mut indices,
                title_rect,
                theme.title_bar,
                theme.corner_radius,
            );
        }

        for child in &window.children {
            let rect = if scroll_y > 0.0 {
                Rect::new(
                    child_rect(child).x,
                    child_rect(child).y - scroll_y,
                    child_rect(child).w,
                    child_rect(child).h,
                )
            } else {
                child_rect(child)
            };
            match child {
                WindowChild::Button(b) => {
                    let color = match b.state {
                        ControlState::Pressed => theme.button_pressed,
                        ControlState::Hover => theme.button_hover,
                        ControlState::Normal => theme.button_bg,
                    };
                    push_quad(
                        &mut vertices,
                        &mut indices,
                        rect,
                        color,
                        theme.corner_radius,
                    );
                }
                WindowChild::Slider(s) => {
                    push_quad(
                        &mut vertices,
                        &mut indices,
                        rect,
                        theme.slider_track,
                        theme.corner_radius,
                    );
                    let thumb = if scroll_y > 0.0 {
                        let t = s.thumb_rect(viewport_width);
                        Rect::new(t.x, t.y - scroll_y, t.w, t.h)
                    } else {
                        s.thumb_rect(viewport_width)
                    };
                    push_quad(
                        &mut vertices,
                        &mut indices,
                        thumb,
                        theme.slider_thumb,
                        theme.corner_radius,
                    );
                }
                WindowChild::Checkbox(c) => {
                    let bg = if c.checked {
                        theme.checkbox_on
                    } else {
                        theme.checkbox_off
                    };
                    push_quad(&mut vertices, &mut indices, rect, bg, theme.corner_radius);
                    if c.checked {
                        let margin = (rect.w.min(rect.h) * 0.25).max(2.0);
                        let check = Rect::new(
                            rect.x + margin,
                            rect.y + margin,
                            rect.w - 2.0 * margin,
                            rect.h - 2.0 * margin,
                        );
                        push_quad(&mut vertices, &mut indices, check, theme.panel_bg, 0.0);
                    }
                }
                WindowChild::Label(l) => {
                    if l.draw_background {
                        push_quad(
                            &mut vertices,
                            &mut indices,
                            rect,
                            theme.title_bar,
                            theme.corner_radius,
                        );
                    }
                    // Text is rendered separately via TextBrush (wgpu_text)
                }
            }
        }

        let index_count = (indices.len() as u32) - index_start;
        ranges.push(WindowDrawRange {
            body_rect: body,
            index_start,
            index_count,
        });
    }

    (vertices, indices, ranges)
}

fn child_rect(child: &WindowChild) -> Rect {
    match child {
        WindowChild::Button(b) => b.rect,
        WindowChild::Slider(s) => s.rect,
        WindowChild::Checkbox(c) => c.rect,
        WindowChild::Label(l) => l.rect,
    }
}
