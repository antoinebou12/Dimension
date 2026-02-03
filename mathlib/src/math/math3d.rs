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
    out.set_zero();
    for j in 0..3 {
        for i in 0..3 {
            out.set(i, out.get(i) + m.get(i, j) * v.get(j));
        }
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
    out.set_zero();
    for j in 0..3 {
        for i in 0..3 {
            out.set(i, out.get(i) + m.get(i, j) * v.get(j));
        }
    }
    out
}

/// 3×3 rotation matrix from Euler angles (roll, pitch, yaw) in radians.
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
