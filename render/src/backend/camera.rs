//! Camera using mathlib orthographic or perspective projection and orbit navigation.
//!
//! The [`Camera3d`] trait abstracts view/projection and viewport so the renderer and picking
//! work with any camera implementation (e.g. orbit, first-person). The default [`Camera`] is an
//! orbit camera.

use mathlib::cg::{look_at_rh, new_orthographic, new_perspective_wgpu, vector3, Perspective3};
use mathlib::math3d::{vector3_cross, Matrix4f};

/// Trait for 3D cameras used by the renderer and picking.
///
/// Implement this for orbit, first-person, or fixed cameras. The renderer calls
/// [`view_matrix`](Self::view_matrix), [`projection_matrix`](Self::projection_matrix), and
/// [`resize`](Self::resize); picking uses [`perspective_params`](Self::perspective_params) and
/// viewport size.
pub trait Camera3d {
    /// View matrix (world to view space, column-major).
    fn view_matrix(&self) -> Matrix4f;
    /// Projection matrix (view to clip, column-major).
    fn projection_matrix(&self) -> Matrix4f;
    /// Update viewport size (e.g. on window resize).
    fn resize(&mut self, width: u32, height: u32);
    /// Perspective parameters for ray casting; `None` for orthographic.
    fn perspective_params(&self) -> Option<Perspective3>;
    /// Viewport width in pixels.
    fn viewport_width(&self) -> f32;
    /// Viewport height in pixels.
    fn viewport_height(&self) -> f32;
}

/// Projection type: orthographic (depth range in view space) or perspective (FOV, positive near/far).
#[derive(Clone, Copy, Debug)]
pub enum Projection {
    /// Orthographic: maps [left,right]×[bottom,top]×[near,far] (view space) to NDC. Use negative near when camera is on +Z looking at origin.
    Orthographic,
    /// Perspective: vertical FOV in radians; near and far are positive (used only for this variant).
    Perspective { fov_y_rad: f32 },
}

/// 2D/3D camera with orbit navigation and configurable orthographic or perspective projection.
#[derive(Clone, Debug)]
pub struct Camera {
    /// Viewport width.
    pub width: f32,
    /// Viewport height.
    pub height: f32,
    /// Projection type (orthographic or perspective).
    pub projection: Projection,
    /// Near plane (view-space z). Orthographic: use negative near (e.g. -10) when camera on +Z looks at origin. Perspective: ignored; projection uses 0.1.
    pub near: f32,
    /// Far plane (view-space z). Orthographic: far > near (e.g. 10). Perspective: ignored; projection uses 100.
    pub far: f32,
    /// Orbit yaw (radians, around Y axis).
    pub orbit_yaw: f32,
    /// Orbit pitch (radians, around X axis).
    pub orbit_pitch: f32,
    /// Distance from target.
    pub orbit_distance: f32,
    /// Look-at target (orbit center). Panned when using [`Self::pan`].
    pub orbit_target: [f32; 3],
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            width: 800.0,
            height: 600.0,
            projection: Projection::Orthographic,
            near: -10.0,
            far: 10.0,
            orbit_yaw: 0.0,
            orbit_pitch: 0.0,
            orbit_distance: 2.5,
            orbit_target: [0.0, 0.0, 0.0],
        }
    }
}

impl Camera {
    /// Create camera for given viewport size with orthographic projection.
    #[must_use]
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            width,
            height,
            projection: Projection::Orthographic,
            near: -10.0,
            far: 10.0,
            orbit_yaw: 0.0,
            orbit_pitch: 0.0,
            orbit_distance: 2.5,
            orbit_target: [0.0, 0.0, 0.0],
        }
    }

    /// Create camera for given viewport size with perspective projection.
    ///
    /// Uses `fov_y_rad` as vertical field-of-view in radians; near and far are 0.1 and 100.
    #[must_use]
    pub fn new_perspective(width: f32, height: f32, fov_y_rad: f32) -> Self {
        Self {
            width,
            height,
            projection: Projection::Perspective { fov_y_rad },
            near: -10.0,
            far: 10.0,
            orbit_yaw: 0.0,
            orbit_pitch: 0.0,
            orbit_distance: 2.5,
            orbit_target: [0.0, 0.0, 0.0],
        }
    }

    /// Resize viewport.
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width as f32;
        self.height = height as f32;
    }

    /// Zoom (adjust distance). Positive = zoom in.
    pub fn zoom(&mut self, delta: f32) {
        self.orbit_distance = (self.orbit_distance - delta).max(0.5).min(20.0);
    }

    /// Add orbit delta (radians). Clamp pitch to avoid flipping.
    pub fn orbit(&mut self, dyaw: f32, dpitch: f32) {
        self.orbit_yaw += dyaw;
        self.orbit_pitch += dpitch;
        const PITCH_LIMIT: f32 = std::f32::consts::FRAC_PI_2 - 0.01;
        self.orbit_pitch = self.orbit_pitch.clamp(-PITCH_LIMIT, PITCH_LIMIT);
    }

    /// Reset orbit to default (yaw 0, pitch 0, distance 2.5, target at origin). Use for a "Reset camera" button.
    pub fn reset_orbit(&mut self) {
        self.orbit_yaw = 0.0;
        self.orbit_pitch = 0.0;
        self.orbit_distance = 2.5;
        self.orbit_target = [0.0, 0.0, 0.0];
    }

    /// Pan the look-at target in the view plane. `dx`, `dy` are in screen space (e.g. pixel deltas).
    /// Scale is proportional to [`Self::orbit_distance`] so pan feels consistent at different zoom levels.
    pub fn pan(&mut self, dx: f32, dy: f32) {
        let y = self.orbit_yaw;
        let p = self.orbit_pitch;
        let d = self.orbit_distance;
        let dir = vector3(p.cos() * y.sin(), p.sin(), p.cos() * y.cos());
        let forward = vector3(-dir.get(0), -dir.get(1), -dir.get(2));
        let world_up = vector3(0.0, 1.0, 0.0);
        let right = vector3_cross(&forward, &world_up);
        let right_len = (right.get(0) * right.get(0)
            + right.get(1) * right.get(1)
            + right.get(2) * right.get(2))
        .sqrt();
        let (rx, ry, rz) = if right_len > 1e-6 {
            (
                right.get(0) / right_len,
                right.get(1) / right_len,
                right.get(2) / right_len,
            )
        } else {
            (1.0, 0.0, 0.0)
        };
        let right_n = vector3(rx, ry, rz);
        let up = vector3_cross(&right_n, &forward);
        let up_len = (up.get(0) * up.get(0) + up.get(1) * up.get(1) + up.get(2) * up.get(2)).sqrt();
        let (ux, uy, uz) = if up_len > 1e-6 {
            (up.get(0) / up_len, up.get(1) / up_len, up.get(2) / up_len)
        } else {
            (0.0, 1.0, 0.0)
        };
        let scale = d * 0.002; // screen delta to world movement
        self.orbit_target[0] += (rx * dx - ux * dy) * scale;
        self.orbit_target[1] += (ry * dx - uy * dy) * scale;
        self.orbit_target[2] += (rz * dx - uz * dy) * scale;
    }

    /// Projection matrix (column-major). NDC -1..1 with aspect ratio.
    #[must_use]
    pub fn projection_matrix(&self) -> Matrix4f {
        let aspect = self.width / self.height.max(1.0);
        match self.projection {
            Projection::Orthographic => {
                let (left, right, bottom, top) = if aspect >= 1.0 {
                    (-aspect, aspect, -1.0, 1.0)
                } else {
                    (-1.0, 1.0, -1.0 / aspect, 1.0 / aspect)
                };
                new_orthographic(left, right, bottom, top, self.near, self.far)
            }
            Projection::Perspective { fov_y_rad } => {
                const PERSP_NEAR: f32 = 0.1;
                const PERSP_FAR: f32 = 100.0;
                new_perspective_wgpu(aspect, fov_y_rad, PERSP_NEAR, PERSP_FAR)
            }
        }
    }

    /// View matrix from orbit (look at [`Self::orbit_target`]).
    #[must_use]
    pub fn view_matrix(&self) -> Matrix4f {
        let y = self.orbit_yaw;
        let p = self.orbit_pitch;
        let d = self.orbit_distance;
        let [tx, ty, tz] = self.orbit_target;
        let target = vector3(tx, ty, tz);
        let eye = vector3(
            tx + d * p.cos() * y.sin(),
            ty + d * p.sin(),
            tz + d * p.cos() * y.cos(),
        );
        let up = vector3(0.0, 1.0, 0.0);
        look_at_rh(&eye, &target, &up)
    }

    /// Perspective parameters for picking. Returns `None` if projection is orthographic.
    /// Picking is supported only for perspective projection in v1.
    #[must_use]
    pub fn perspective_params(&self) -> Option<Perspective3> {
        match self.projection {
            Projection::Perspective { fov_y_rad } => {
                let aspect = self.width / self.height.max(1.0);
                Some(Perspective3::new(aspect, fov_y_rad, 0.1, 100.0))
            }
            Projection::Orthographic => None,
        }
    }
}

impl Camera3d for Camera {
    fn view_matrix(&self) -> Matrix4f {
        Camera::view_matrix(self)
    }

    fn projection_matrix(&self) -> Matrix4f {
        Camera::projection_matrix(self)
    }

    fn resize(&mut self, width: u32, height: u32) {
        Camera::resize(self, width, height);
    }

    fn perspective_params(&self) -> Option<Perspective3> {
        Camera::perspective_params(self)
    }

    fn viewport_width(&self) -> f32 {
        self.width
    }

    fn viewport_height(&self) -> f32 {
        self.height
    }
}
