//! Rigid body: position, quaternion orientation, velocity, inertia, collision shape.
//!
//! Unlike particle-based shape-matching, [`RigidBody`] tracks orientation as a
//! quaternion and angular velocity explicitly. The XPBD solver applies both
//! positional and angular corrections.

use mathlib::math3d_raw::{QUAT_IDENTITY, mat3_mul, mat3_transpose, quat_normalize, quat_to_mat3};

/// Collision shape in body-local coordinates.
///
/// Wraps shape parameters; actual intersection tests use the collision crate.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CollisionShape {
    /// Sphere centered at body origin with the given radius.
    Sphere { radius: f32 },
    /// Axis-aligned box (half-extents along each axis).
    Box { half_extents: [f32; 3] },
    /// Capsule along the Y axis (radius + half-height).
    Capsule { radius: f32, half_height: f32 },
}

impl CollisionShape {
    /// Compute a conservative world-space AABB half-extents for any shape.
    ///
    /// This is rotation-independent (bounding sphere) for simplicity;
    /// a tighter OBB-based bound can be added later.
    #[must_use]
    pub fn bounding_radius(&self) -> f32 {
        match self {
            Self::Sphere { radius } => *radius,
            Self::Box { half_extents } => {
                let [hx, hy, hz] = *half_extents;
                (hx * hx + hy * hy + hz * hz).sqrt()
            }
            Self::Capsule {
                radius,
                half_height,
            } => radius + half_height,
        }
    }
}

/// Rigid body with position, orientation, velocity, and inertia.
///
/// # Quaternion convention
///
/// Orientation quaternion is stored as `[w, x, y, z]` (scalar-first).
///
/// # Inertia tensor
///
/// `inv_inertia` is the **body-frame** inverse inertia tensor, stored as a
/// row-major 3×3 matrix. For axis-aligned shapes, this is diagonal.
/// The world-frame inverse inertia is computed at runtime: `I⁻¹_world = R · I⁻¹_body · Rᵀ`.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RigidBody {
    // ── Linear state ────────────────────────────────────────────
    /// Center of mass position.
    pub x: [f32; 3],
    /// Linear velocity.
    pub v: [f32; 3],
    /// Inverse mass (0 = kinematic / fixed).
    pub inv_mass: f32,

    // ── Angular state ───────────────────────────────────────────
    /// Orientation quaternion `[w, x, y, z]`.
    pub q: [f32; 4],
    /// Angular velocity (world frame).
    pub omega: [f32; 3],
    /// Inverse inertia tensor in **body frame** (row-major 3×3).
    pub inv_inertia: [f32; 9],

    // ── Collision ───────────────────────────────────────────────
    /// Collision shape in body-local coordinates.
    pub shape: CollisionShape,

    // ── Metadata ────────────────────────────────────────────────
    /// Phase for collision grouping.
    pub phase: u32,
    /// Whether this body is currently sleeping.
    pub sleeping: bool,

    // ── Predicted state (used during solve) ─────────────────────
    /// Predicted position (set during predict step).
    #[cfg_attr(feature = "serde", serde(skip))]
    pub predicted_x: [f32; 3],
    /// Predicted orientation (set during predict step).
    #[cfg_attr(feature = "serde", serde(skip))]
    pub predicted_q: [f32; 4],
}

impl RigidBody {
    /// Creates a dynamic rigid body at the given position with the given mass and shape.
    ///
    /// The orientation is identity (no rotation), and angular velocity is zero.
    /// The inertia tensor is computed from the shape and mass.
    ///
    /// # Examples
    ///
    /// ```
    /// use physics::body::{RigidBody, CollisionShape};
    /// let rb = RigidBody::new([0.0, 5.0, 0.0], 1.0, CollisionShape::Sphere { radius: 0.5 });
    /// assert!(!rb.is_kinematic());
    /// ```
    #[must_use]
    pub fn new(position: [f32; 3], mass: f32, shape: CollisionShape) -> Self {
        let inv_mass = if mass > 0.0 { 1.0 / mass } else { 0.0 };
        let inv_inertia = compute_inv_inertia(&shape, mass);
        Self {
            x: position,
            v: [0.0; 3],
            inv_mass,
            q: QUAT_IDENTITY,
            omega: [0.0; 3],
            inv_inertia,
            shape,
            phase: 0,
            sleeping: false,
            predicted_x: position,
            predicted_q: QUAT_IDENTITY,
        }
    }

    /// Creates a kinematic (fixed) rigid body.
    #[must_use]
    pub fn kinematic(position: [f32; 3], shape: CollisionShape) -> Self {
        Self {
            x: position,
            v: [0.0; 3],
            inv_mass: 0.0,
            q: QUAT_IDENTITY,
            omega: [0.0; 3],
            inv_inertia: [0.0; 9],
            shape,
            phase: 0,
            sleeping: false,
            predicted_x: position,
            predicted_q: QUAT_IDENTITY,
        }
    }

    /// Returns `true` if this body is kinematic (infinite mass).
    #[must_use]
    pub fn is_kinematic(&self) -> bool {
        self.inv_mass <= 0.0
    }

    /// Sets the phase identifier.
    #[must_use]
    pub fn with_phase(mut self, phase: u32) -> Self {
        self.phase = phase;
        self
    }

    /// Sets the orientation quaternion.
    #[must_use]
    pub fn with_orientation(mut self, q: [f32; 4]) -> Self {
        self.q = quat_normalize(&q);
        self.predicted_q = self.q;
        self
    }

    /// Compute world-frame inverse inertia: `R · I⁻¹_body · Rᵀ`.
    #[must_use]
    pub fn inv_inertia_world(&self) -> [f32; 9] {
        let r = quat_to_mat3(&self.q);
        let rt = mat3_transpose(&r);
        let tmp = mat3_mul(&r, &self.inv_inertia);
        mat3_mul(&tmp, &rt)
    }

    /// World-space AABB min/max for broad phase.
    #[must_use]
    pub fn aabb_min_max(&self) -> ([f32; 3], [f32; 3]) {
        let r = self.shape.bounding_radius();
        (
            [self.x[0] - r, self.x[1] - r, self.x[2] - r],
            [self.x[0] + r, self.x[1] + r, self.x[2] + r],
        )
    }
}

impl Default for RigidBody {
    fn default() -> Self {
        Self::new([0.0; 3], 1.0, CollisionShape::Sphere { radius: 0.5 })
    }
}

/// Compute the inverse inertia tensor for a shape given its mass.
fn compute_inv_inertia(shape: &CollisionShape, mass: f32) -> [f32; 9] {
    if mass <= 0.0 {
        return [0.0; 9];
    }
    let mut inv = [0.0_f32; 9];
    match shape {
        CollisionShape::Sphere { radius } => {
            // I = (2/5) m r²
            let i = 2.0 / 5.0 * mass * radius * radius;
            if i > 0.0 {
                let inv_i = 1.0 / i;
                inv[0] = inv_i;
                inv[4] = inv_i;
                inv[8] = inv_i;
            }
        }
        CollisionShape::Box { half_extents } => {
            let [hx, hy, hz] = *half_extents;
            let sx = 2.0 * hx;
            let sy = 2.0 * hy;
            let sz = 2.0 * hz;
            // I_xx = (1/12) m (sy² + sz²), etc.
            let ix = mass / 12.0 * (sy * sy + sz * sz);
            let it = mass / 12.0 * (sx * sx + sz * sz);
            let iz = mass / 12.0 * (sx * sx + sy * sy);
            if ix > 0.0 {
                inv[0] = 1.0 / ix;
            }
            if it > 0.0 {
                inv[4] = 1.0 / it;
            }
            if iz > 0.0 {
                inv[8] = 1.0 / iz;
            }
        }
        CollisionShape::Capsule {
            radius,
            half_height,
        } => {
            // Approximate as cylinder for inertia
            let h = 2.0 * half_height;
            let r2 = radius * radius;
            let ix = mass * (3.0 * r2 + h * h) / 12.0;
            let it = mass * r2 / 2.0;
            let iz = ix;
            if ix > 0.0 {
                inv[0] = 1.0 / ix;
            }
            if it > 0.0 {
                inv[4] = 1.0 / it;
            }
            if iz > 0.0 {
                inv[8] = 1.0 / iz;
            }
        }
    }
    inv
}
