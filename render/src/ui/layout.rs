//! Screen-space rectangles and layout helpers for UI.

/// Axis-aligned rectangle in pixel coordinates. Origin (0, 0) is top-left; y increases downward.
#[derive(Clone, Copy, Debug, Default)]
pub struct Rect {
    /// Left edge (x).
    pub x: f32,
    /// Top edge (y).
    pub y: f32,
    /// Width.
    pub w: f32,
    /// Height.
    pub h: f32,
}

impl Rect {
    /// New rectangle from position and size.
    #[must_use]
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    /// Right edge (x + w).
    #[must_use]
    pub fn right(&self) -> f32 {
        self.x + self.w
    }

    /// Bottom edge (y + h).
    #[must_use]
    pub fn bottom(&self) -> f32 {
        self.y + self.h
    }

    /// True if the point (px, py) is inside this rect (inclusive edges).
    #[must_use]
    pub fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && px <= self.right() && py >= self.y && py <= self.bottom()
    }
}

/// Vertical layout: yields rects for a stack of rows (e.g. buttons, sliders) with fixed row height and spacing.
///
/// Use when building dynamic lists so you don't hand-calculate `y` for each row.
///
/// # Examples
///
/// ```
/// # use crate::ui::layout::{Rect, VerticalLayout};
/// let body = Rect::new(10.0, 50.0, 200.0, 400.0);
/// let mut layout = VerticalLayout::new(body, 24.0, 4.0);
/// let row1 = layout.next_rect(150.0);  // width 150
/// let row2 = layout.next_rect(150.0);
/// ```
#[derive(Clone, Debug)]
pub struct VerticalLayout {
    /// Next top-edge y in the layout (increases after each row).
    next_y: f32,
    /// Height of each row.
    row_height: f32,
    /// Vertical gap between rows.
    spacing: f32,
    /// Bounds: x and width come from body; y advances from body.y.
    body: Rect,
}

impl VerticalLayout {
    /// New vertical layout inside the given body rect. Rows start at `body.y`; each row is `row_height` tall with `spacing` below it.
    #[must_use]
    pub fn new(body: Rect, row_height: f32, spacing: f32) -> Self {
        Self {
            next_y: body.y,
            row_height,
            spacing,
            body,
        }
    }

    /// Next row rect with the given width, left-aligned to the body. Width is clamped to body width.
    #[must_use]
    pub fn next_rect(&mut self, width: f32) -> Rect {
        let w = width.min(self.body.w);
        let r = Rect::new(self.body.x, self.next_y, w, self.row_height);
        self.next_y += self.row_height + self.spacing;
        r
    }

    /// Current content height used so far (from body.y to the bottom of the last row).
    #[must_use]
    pub fn content_height(&self) -> f32 {
        (self.next_y - self.body.y).max(0.0)
    }
}

/// Returns an iterator of rects for a vertical stack: `count` rows of `row_height` with `spacing` between, starting at (origin_x, origin_y).
/// Row width is `row_width`; use for building fixed-size rows without mutating a layout.
#[must_use]
pub fn vertical_stack(
    origin_x: f32,
    origin_y: f32,
    row_width: f32,
    row_height: f32,
    spacing: f32,
    count: usize,
) -> impl Iterator<Item = Rect> {
    (0..count).map(move |i| {
        let y = origin_y + (i as f32) * (row_height + spacing);
        Rect::new(origin_x, y, row_width, row_height)
    })
}
