//! 3D math types and transformations.
//!
//! This module provides type aliases and functions for 3D graphics and geometry:
//!
//! - **Vectors**: [`Vector3f`], [`Vector4f`] (column vectors of `f32`)
//! - **Matrices**: [`Matrix3f`], [`Matrix4f`] (3×3 and 4×4 transformation matrices)
//! - **Points**: [`Point3`] (distinct from vectors; represents a location in space)
//!
//! # Type Aliases
//!
//! All 3D types use `f32` to match GPU precision:
//!
//! | Type | Description |
//! |------|-------------|
//! | `Vector3f` | 3D vector (x, y, z) |
//! | `Vector4f` | 4D homogeneous vector (x, y, z, w) |
//! | `Matrix3f` | 3×3 rotation/scale matrix |
//! | `Matrix4f` | 4×4 transformation matrix (rotation, scale, translation) |
//!
//! # Key Functions
//!
//! - [`make_rotation`]: Create a 3×3 rotation matrix from Euler angles
//! - [`matrix4f_inverse`]: Invert a 4×4 matrix
//! - [`matrix4_mul_vector3`]: Transform a point by a 4×4 matrix
//! - [`transform_vector`]: Transform a direction (ignores translation)
//! - [`center`]: Compute centroid of points
//! - [`OrthonormalBasis`]: Build an orthonormal frame from a vector (e.g. normal)
//!
//! # Example
//!
//! ```
//! use mathlib::{Matrix4f, Vector3f, make_rotation, matrix4_mul_vector3};
//!
//! let rotation = make_rotation(0.0, std::f32::consts::FRAC_PI_2, 0.0);
//! let mut point = Vector3f::with_capacity(3);
//! point.data_mut().copy_from_slice(&[1.0, 0.0, 0.0]);
//! // Rotating (1, 0, 0) by 90° around Y gives approximately (0, 0, -1)
//! ```

use crate::matrix::Matrix;
use crate::types::Storage;
use crate::vector::Vector;
use std::ops::{Add, Sub};

/// 3D float vector (column).
pub type Vector3f = Vector<f32>;
/// 4D float vector (homogeneous).
pub type Vector4f = Vector<f32>;
/// 3×3 float matrix.
pub type Matrix3f = Matrix<f32>;
/// 4×4 float matrix (homogeneous transform).
pub type Matrix4f = Matrix<f32>;

/// 3D point: a location in space. Distinct from vectors (translations).
/// Point + Vector = Point; Point - Point = Vector.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Point3 {
    /// Coordinates (x, y, z).
    pub coords: Vector3f,
}

impl PartialEq for Point3 {
    fn eq(&self, other: &Self) -> bool {
        self.coords.get(0) == other.coords.get(0)
            && self.coords.get(1) == other.coords.get(1)
            && self.coords.get(2) == other.coords.get(2)
    }
}

impl Point3 {
    /// Creates a point (x, y, z).
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        let mut coords = Vector3f::with_capacity(3);
        coords.set(0, x);
        coords.set(1, y);
        coords.set(2, z);
        Self { coords }
    }

    /// Creates a point from a 3D vector.
    pub fn from_coordinates(v: Vector3f) -> Self {
        assert!(v.rows() == 3);
        Self { coords: v }
    }

    /// Origin (0, 0, 0).
    pub fn origin() -> Self {
        let mut coords = Vector3f::with_capacity(3);
        coords.set_zero();
        Self { coords }
    }
}

impl Add<Vector3f> for Point3 {
    type Output = Point3;

    fn add(self, v: Vector3f) -> Point3 {
        assert!(v.rows() == 3);
        let mut coords = Vector3f::with_capacity(3);
        for i in 0..3 {
            coords.set(i, self.coords.get(i) + v.get(i));
        }
        Point3 { coords }
    }
}

impl Add<Vector3f> for &Point3 {
    type Output = Point3;

    fn add(self, v: Vector3f) -> Point3 {
        assert!(v.rows() == 3);
        let mut coords = Vector3f::with_capacity(3);
        for i in 0..3 {
            coords.set(i, self.coords.get(i) + v.get(i));
        }
        Point3 { coords }
    }
}

impl Sub for Point3 {
    type Output = Vector3f;

    fn sub(self, other: Point3) -> Vector3f {
        let mut out = Vector3f::with_capacity(3);
        for i in 0..3 {
            out.set(i, self.coords.get(i) - other.coords.get(i));
        }
        out
    }
}

impl Sub for &Point3 {
    type Output = Vector3f;

    fn sub(self, other: &Point3) -> Vector3f {
        let mut out = Vector3f::with_capacity(3);
        for i in 0..3 {
            out.set(i, self.coords.get(i) - other.coords.get(i));
        }
        out
    }
}

/// Midpoint of two points: (a + b) / 2.
#[must_use]
pub fn center(a: &Point3, b: &Point3) -> Point3 {
    let mut coords = Vector3f::with_capacity(3);
    for i in 0..3 {
        coords.set(i, (a.coords.get(i) + b.coords.get(i)) * 0.5);
    }
    Point3 { coords }
}

// --- Homogeneous coordinates -------------------------------------------------

/// Point in homogeneous form: [x, y, z, 1].
#[must_use]
pub fn point_to_homogeneous(p: &Point3) -> Vector4f {
    let mut out = Vector4f::with_capacity(4);
    out.set(0, p.coords.get(0));
    out.set(1, p.coords.get(1));
    out.set(2, p.coords.get(2));
    out.set(3, 1.0);
    out
}

/// Vector in homogeneous form: [x, y, z, 0].
#[must_use]
pub fn vector_to_homogeneous(v: &Vector3f) -> Vector4f {
    assert!(v.rows() == 3);
    let mut out = Vector4f::with_capacity(4);
    out.set(0, v.get(0));
    out.set(1, v.get(1));
    out.set(2, v.get(2));
    out.set(3, 0.0);
    out
}

/// Point from homogeneous coords: divide x,y,z by w; returns None if w ≈ 0.
pub fn from_homogeneous_point(v: &Vector4f) -> Option<Point3> {
    assert!(v.rows() == 4);
    let w = v.get(3);
    if w.abs() < 1e-20 {
        return None;
    }
    let mut coords = Vector3f::with_capacity(3);
    coords.set(0, v.get(0) / w);
    coords.set(1, v.get(1) / w);
    coords.set(2, v.get(2) / w);
    Some(Point3 { coords })
}

/// Vector from homogeneous coords: drop last component (typically 0).
#[must_use]
pub fn from_homogeneous_vector(v: &Vector4f) -> Vector3f {
    assert!(v.rows() == 4);
    let mut out = Vector3f::with_capacity(3);
    out.set(0, v.get(0));
    out.set(1, v.get(1));
    out.set(2, v.get(2));
    out
}

pub fn matrix4f_inverse(m: &Matrix4f) -> Matrix4f {
    let mut out = Matrix4f::with_storage(4, 4, Storage::Column);
    out.set_zero();
    for i in 0..3 {
        for j in 0..3 {
            out.set(i, j, m.get(j, i));
        }
    }
    let tx = m.get(0, 3);
    let ty = m.get(1, 3);
    let tz = m.get(2, 3);
    out.set(
        0,
        3,
        -(out.get(0, 0) * tx + out.get(0, 1) * ty + out.get(0, 2) * tz),
    );
    out.set(
        1,
        3,
        -(out.get(1, 0) * tx + out.get(1, 1) * ty + out.get(1, 2) * tz),
    );
    out.set(
        2,
        3,
        -(out.get(2, 0) * tx + out.get(2, 1) * ty + out.get(2, 2) * tz),
    );
    out.set(3, 3, 1.0);
    out
}

/// Inverse of a 3×3 orthogonal matrix (transpose).
pub fn matrix3f_inverse(m: &Matrix3f) -> Matrix3f {
    m.transpose()
}

/// Transform a 3D point (homogeneous w = 1). Translation is applied.
pub fn matrix4_mul_vector3(m: &Matrix4f, v: &Vector3f) -> Vector3f {
    assert!(v.rows() == 3);
    let mut out = Vector3f::with_capacity(3);
    let v_data = v.data();
    for i in 0..3 {
        let row_i = [m.get(i, 0), m.get(i, 1), m.get(i, 2)];
        out.set(i, crate::cpu::dot_f32(&row_i, v_data));
    }
    out.set(0, out.get(0) + m.get(0, 3));
    out.set(1, out.get(1) + m.get(1, 3));
    out.set(2, out.get(2) + m.get(2, 3));
    out
}

/// Transform a 3D direction vector (3×3 part only; translation is not applied).
#[must_use]
pub fn transform_vector(m: &Matrix4f, v: &Vector3f) -> Vector3f {
    assert!(v.rows() == 3);
    let mut out = Vector3f::with_capacity(3);
    let v_data = v.data();
    for i in 0..3 {
        let row_i = [m.get(i, 0), m.get(i, 1), m.get(i, 2)];
        out.set(i, crate::cpu::dot_f32(&row_i, v_data));
    }
    out
}

/// Cross product of two 3D vectors: a × b.
///
/// # Examples
///
/// ```
/// # use mathlib::math3d::{Vector3f, vector3_cross};
/// # use mathlib::cg::vector3;
/// let a = vector3(1.0, 0.0, 0.0);
/// let b = vector3(0.0, 1.0, 0.0);
/// let c = vector3_cross(&a, &b);
/// assert!((c.get(2) - 1.0).abs() < 1e-5); // a × b = (0, 0, 1)
/// ```
#[must_use]
pub fn vector3_cross(a: &Vector3f, b: &Vector3f) -> Vector3f {
    let mut out = Vector3f::with_capacity(3);
    out.set(0, a.get(1) * b.get(2) - a.get(2) * b.get(1));
    out.set(1, a.get(2) * b.get(0) - a.get(0) * b.get(2));
    out.set(2, a.get(0) * b.get(1) - a.get(1) * b.get(0));
    out
}

// --- Orthonormal basis -------------------------------------------------------

/// Trait for computing an orthonormal basis from a single vector.
///
/// Given a vector (typically a unit normal), this yields vectors that together
/// form an orthonormal frame. Useful for building tangent frames, TBN matrices,
/// or any basis where one axis is given.
///
/// # Panics
///
/// Implementations may panic if the vector has the wrong length (e.g. [`Vector3f`]
/// must have 3 elements).
///
/// # Examples
///
/// ```
/// # use mathlib::math3d::{Vector3f, OrthonormalBasis};
/// # use mathlib::cg::vector3;
/// let n = vector3(0.0, 1.0, 0.0); // unit Y
/// let [t, b] = n.clone().orthonormal_basis();
/// // t, b are unit and orthogonal to n and to each other
/// assert!((t.norm() - 1.0).abs() < 1e-5);
/// assert!((b.norm() - 1.0).abs() < 1e-5);
/// assert!(t.dot(&n).abs() < 1e-5);
/// assert!(b.dot(&n).abs() < 1e-5);
/// assert!(t.dot(&b).abs() < 1e-5);
/// ```
pub trait OrthonormalBasis: Sized {
    /// Type of the array of orthonormal vectors that complete the basis with `self`.
    type Basis;

    /// Returns the vectors that, together with `self`, form an orthonormal basis.
    ///
    /// For a unit input, the returned vectors are unit and mutually orthogonal.
    #[must_use]
    fn orthonormal_basis(self) -> Self::Basis;

    /// Returns a single vector orthogonal to `self` with unit length (when `self` is unit).
    #[must_use]
    fn orthonormal_vector(self) -> Self;
}

impl OrthonormalBasis for Vector3f {
    type Basis = [Vector3f; 2];

    // Branchless construction from Pixar: https://graphics.pixar.com/library/OrthonormalB/paper.pdf
    fn orthonormal_basis(self) -> [Vector3f; 2] {
        assert!(self.rows() == 3, "Vector3f must have 3 elements");
        let x = self.get(0);
        let y = self.get(1);
        let z = self.get(2);
        let sign = f32::copysign(1.0, z);
        let a = -1.0 / (sign + z);
        let b = x * y * a;

        let mut t = Vector3f::with_capacity(3);
        t.set(0, 1.0 + sign * x * x * a);
        t.set(1, sign * b);
        t.set(2, -sign * x);

        let mut b_vec = Vector3f::with_capacity(3);
        b_vec.set(0, b);
        b_vec.set(1, sign + y * y * a);
        b_vec.set(2, -y);

        [t, b_vec]
    }

    fn orthonormal_vector(self) -> Vector3f {
        assert!(self.rows() == 3, "Vector3f must have 3 elements");
        let x = self.get(0);
        let y = self.get(1);
        let z = self.get(2);
        let sign = f32::copysign(1.0, z);
        let a = -1.0 / (sign + z);
        let b = x * y * a;
        let mut out = Vector3f::with_capacity(3);
        out.set(0, b);
        out.set(1, sign + y * y * a);
        out.set(2, -y);
        out
    }
}

/// Wraps an angle in radians to the range `(-π, π]`.
#[must_use]
pub fn wrap_angle_to_pi(angle: f32) -> f32 {
    use std::f32::consts::PI;
    let two_pi = 2.0 * PI;
    let mut a = angle % two_pi;
    if a > PI {
        a -= two_pi;
    } else if a <= -PI {
        a += two_pi;
    }
    a
}

/// Extracts Euler angles (roll, pitch, yaw) in radians from a 3×3 rotation matrix.
///
/// Assumes composition R = Rz * Ry * Rx (XYZ order), matching [`make_rotation`].
/// Returns `(roll, pitch, yaw)` in radians.
///
/// # Panics
///
/// Panics if R is not a valid rotation matrix (orthonormal).
#[must_use]
pub fn rotation_matrix_to_euler_xyz(r: &Matrix3f) -> (f32, f32, f32) {
    let sy = (r.get(0, 0) * r.get(0, 0) + r.get(1, 0) * r.get(1, 0)).sqrt();
    let singular = sy < 1e-6;
    let (roll, pitch, yaw) = if singular {
        let roll = r.get(1, 2).atan2(r.get(2, 2));
        let pitch = (-r.get(2, 0)).clamp(-1.0, 1.0).asin();
        (roll, pitch, 0.0)
    } else {
        let roll = r.get(2, 1).atan2(r.get(2, 2));
        let pitch = (-r.get(2, 0)).clamp(-1.0, 1.0).asin();
        let yaw = r.get(1, 0).atan2(r.get(0, 0));
        (roll, pitch, yaw)
    };
    (roll, pitch, yaw)
}

/// Returns Euler angles close to `reference` for stable reroot/IK.
///
/// Wraps each angle to stay near the reference (avoids 2π jumps).
#[must_use]
pub fn euler_angles_close_to(
    euler: (f32, f32, f32),
    reference: (f32, f32, f32),
) -> (f32, f32, f32) {
    use std::f32::consts::PI;
    let wrap_near = |a: f32, r: f32| -> f32 {
        let mut d = a - r;
        while d > PI {
            d -= 2.0 * PI;
        }
        while d < -PI {
            d += 2.0 * PI;
        }
        r + d
    };
    (
        wrap_near(euler.0, reference.0),
        wrap_near(euler.1, reference.1),
        wrap_near(euler.2, reference.2),
    )
}

/// 3×3 rotation matrix from Euler angles (roll, pitch, yaw) in radians.
/// Composition is R = Rz * Ry * Rx (XYZ order).
pub fn make_rotation(x: f32, y: f32, z: f32) -> Matrix3f {
    let cx = x.cos();
    let sx = x.sin();
    let cy = y.cos();
    let sy = y.sin();
    let cz = z.cos();
    let sz = z.sin();
    let mut r = Matrix3f::with_storage(3, 3, Storage::Column);
    r.set(0, 0, cy * cz);
    r.set(0, 1, -cx * sz + sx * sy * cz);
    r.set(0, 2, sx * sz + cx * sy * cz);
    r.set(1, 0, cy * sz);
    r.set(1, 1, cx * cz + sx * sy * sz);
    r.set(1, 2, -sx * cz + cx * sy * sz);
    r.set(2, 0, -sy);
    r.set(2, 1, sx * cy);
    r.set(2, 2, cx * cy);
    r
}
