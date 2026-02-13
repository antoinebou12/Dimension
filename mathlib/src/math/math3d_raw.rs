//! Raw array-based 3D math: 3×3 matrices and 3-vectors as `[f32; 9]` and `[f32; 3]`.
//!
//! Thin standalone routines for physics and other f32-heavy workloads that prefer
//! fixed-size arrays over heap-allocated [`Matrix3f`] / [`Vector3f`]. All 3×3
//! matrices are stored in **row-major** order:
//!
//! ```text
//! [m00 m01 m02]
//! [m10 m11 m12]     →  [m00, m01, m02, m10, m11, m12, m20, m21, m22]
//! [m20 m21 m22]
//! ```
//!
//! [`Matrix3f`]: crate::math3d::Matrix3f
//! [`Vector3f`]: crate::math3d::Vector3f

/// Determinant of a 3×3 matrix (row-major `[f32; 9]`).
///
/// # Examples
///
/// ```
/// # use mathlib::math3d_raw::mat3_det;
/// let identity = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
/// assert!((mat3_det(&identity) - 1.0).abs() < 1e-6);
/// ```
#[must_use]
pub fn mat3_det(m: &[f32; 9]) -> f32 {
    m[0] * (m[4] * m[8] - m[5] * m[7]) - m[1] * (m[3] * m[8] - m[5] * m[6])
        + m[2] * (m[3] * m[7] - m[4] * m[6])
}

/// Inverse of a 3×3 matrix (row-major). Returns `None` if singular.
///
/// # Examples
///
/// ```
/// # use mathlib::math3d_raw::mat3_inverse;
/// let m = [2.0, 0.0, 0.0, 0.0, 3.0, 0.0, 0.0, 0.0, 4.0];
/// let inv = mat3_inverse(&m).unwrap();
/// assert!((inv[0] - 0.5).abs() < 1e-6);
/// assert!((inv[4] - 1.0 / 3.0).abs() < 1e-6);
/// ```
#[must_use]
pub fn mat3_inverse(m: &[f32; 9]) -> Option<[f32; 9]> {
    let det = mat3_det(m);
    if det.abs() < 1e-12 {
        return None;
    }
    let inv_det = 1.0 / det;
    Some([
        (m[4] * m[8] - m[5] * m[7]) * inv_det,
        (m[2] * m[7] - m[1] * m[8]) * inv_det,
        (m[1] * m[5] - m[2] * m[4]) * inv_det,
        (m[5] * m[6] - m[3] * m[8]) * inv_det,
        (m[0] * m[8] - m[2] * m[6]) * inv_det,
        (m[2] * m[3] - m[0] * m[5]) * inv_det,
        (m[3] * m[7] - m[4] * m[6]) * inv_det,
        (m[1] * m[6] - m[0] * m[7]) * inv_det,
        (m[0] * m[4] - m[1] * m[3]) * inv_det,
    ])
}

/// Multiply two 3×3 matrices: `result = a * b` (row-major).
#[must_use]
pub fn mat3_mul(a: &[f32; 9], b: &[f32; 9]) -> [f32; 9] {
    [
        a[0] * b[0] + a[1] * b[3] + a[2] * b[6],
        a[0] * b[1] + a[1] * b[4] + a[2] * b[7],
        a[0] * b[2] + a[1] * b[5] + a[2] * b[8],
        a[3] * b[0] + a[4] * b[3] + a[5] * b[6],
        a[3] * b[1] + a[4] * b[4] + a[5] * b[7],
        a[3] * b[2] + a[4] * b[5] + a[5] * b[8],
        a[6] * b[0] + a[7] * b[3] + a[8] * b[6],
        a[6] * b[1] + a[7] * b[4] + a[8] * b[7],
        a[6] * b[2] + a[7] * b[5] + a[8] * b[8],
    ]
}

/// Transpose a 3×3 row-major matrix.
#[must_use]
pub fn mat3_transpose(m: &[f32; 9]) -> [f32; 9] {
    [m[0], m[3], m[6], m[1], m[4], m[7], m[2], m[5], m[8]]
}

/// Multiply a 3×3 row-major matrix by a 3-vector: `result = M * v`.
#[must_use]
pub fn mat3_mul_vec(m: &[f32; 9], v: &[f32; 3]) -> [f32; 3] {
    [
        m[0] * v[0] + m[1] * v[1] + m[2] * v[2],
        m[3] * v[0] + m[4] * v[1] + m[5] * v[2],
        m[6] * v[0] + m[7] * v[1] + m[8] * v[2],
    ]
}

/// 3×3 identity matrix (row-major).
pub const MAT3_IDENTITY: [f32; 9] = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];

/// Frobenius norm of a 3×3 matrix: `sqrt(Σ m_ij²)`.
#[must_use]
pub fn mat3_frobenius_norm(m: &[f32; 9]) -> f32 {
    let mut sum = 0.0_f32;
    for &val in m {
        sum += val * val;
    }
    sum.sqrt()
}

/// Adjugate (cofactor transpose) of a 3×3 matrix.
///
/// For computing `∂det(F)/∂F = adj(F)`.
#[must_use]
pub fn mat3_adjugate(m: &[f32; 9]) -> [f32; 9] {
    [
        m[4] * m[8] - m[5] * m[7],
        m[2] * m[7] - m[1] * m[8],
        m[1] * m[5] - m[2] * m[4],
        m[5] * m[6] - m[3] * m[8],
        m[0] * m[8] - m[2] * m[6],
        m[2] * m[3] - m[0] * m[5],
        m[3] * m[7] - m[4] * m[6],
        m[1] * m[6] - m[0] * m[7],
        m[0] * m[4] - m[1] * m[3],
    ]
}

// ── Quaternion helpers ──────────────────────────────────────────────

/// Rotate a 3-vector by a unit quaternion `q = [w, x, y, z]`.
///
/// Uses the formula `v' = q * [0, v] * q⁻¹` (conjugate for unit q).
///
/// # Examples
///
/// ```
/// # use mathlib::math3d_raw::quat_rotate_vec;
/// let identity = [1.0, 0.0, 0.0, 0.0];
/// let v = [1.0, 2.0, 3.0];
/// let result = quat_rotate_vec(&identity, &v);
/// assert!((result[0] - 1.0).abs() < 1e-6);
/// ```
#[must_use]
pub fn quat_rotate_vec(q: &[f32; 4], v: &[f32; 3]) -> [f32; 3] {
    let (w, x, y, z) = (q[0], q[1], q[2], q[3]);
    // t = 2 * (q_vec × v)
    let tx = 2.0 * (y * v[2] - z * v[1]);
    let ty = 2.0 * (z * v[0] - x * v[2]);
    let tz = 2.0 * (x * v[1] - y * v[0]);
    // result = v + w * t + q_vec × t
    [
        v[0] + w * tx + (y * tz - z * ty),
        v[1] + w * ty + (z * tx - x * tz),
        v[2] + w * tz + (x * ty - y * tx),
    ]
}

/// Multiply two quaternions `a * b`. Both are `[w, x, y, z]`.
#[must_use]
pub fn quat_mul(a: &[f32; 4], b: &[f32; 4]) -> [f32; 4] {
    [
        a[0] * b[0] - a[1] * b[1] - a[2] * b[2] - a[3] * b[3],
        a[0] * b[1] + a[1] * b[0] + a[2] * b[3] - a[3] * b[2],
        a[0] * b[2] - a[1] * b[3] + a[2] * b[0] + a[3] * b[1],
        a[0] * b[3] + a[1] * b[2] - a[2] * b[1] + a[3] * b[0],
    ]
}

/// Conjugate of quaternion `[w, x, y, z]` → `[w, -x, -y, -z]`.
#[must_use]
pub fn quat_conjugate(q: &[f32; 4]) -> [f32; 4] {
    [q[0], -q[1], -q[2], -q[3]]
}

/// Normalize a quaternion to unit length.
#[must_use]
pub fn quat_normalize(q: &[f32; 4]) -> [f32; 4] {
    let len = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
    if len < 1e-12 {
        return [1.0, 0.0, 0.0, 0.0];
    }
    let inv = 1.0 / len;
    [q[0] * inv, q[1] * inv, q[2] * inv, q[3] * inv]
}

/// Identity quaternion `[1, 0, 0, 0]`.
pub const QUAT_IDENTITY: [f32; 4] = [1.0, 0.0, 0.0, 0.0];

/// Convert a unit quaternion `[w, x, y, z]` to a 3×3 rotation matrix (row-major).
#[must_use]
pub fn quat_to_mat3(q: &[f32; 4]) -> [f32; 9] {
    let (w, x, y, z) = (q[0], q[1], q[2], q[3]);
    let x2 = x + x;
    let y2 = y + y;
    let z2 = z + z;
    let xx = x * x2;
    let xy = x * y2;
    let xz = x * z2;
    let yy = y * y2;
    let yz = y * z2;
    let zz = z * z2;
    let wx = w * x2;
    let wy = w * y2;
    let wz = w * z2;
    [
        1.0 - (yy + zz),
        xy - wz,
        xz + wy,
        xy + wz,
        1.0 - (xx + zz),
        yz - wx,
        xz - wy,
        yz + wx,
        1.0 - (xx + yy),
    ]
}

// ── Vector helpers ──────────────────────────────────────────────────

/// 3-vector dot product.
#[inline]
#[must_use]
pub fn vec3_dot(a: &[f32; 3], b: &[f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// 3-vector cross product.
#[inline]
#[must_use]
pub fn vec3_cross(a: &[f32; 3], b: &[f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Squared length of a 3-vector.
#[inline]
#[must_use]
pub fn vec3_length_sq(v: &[f32; 3]) -> f32 {
    v[0] * v[0] + v[1] * v[1] + v[2] * v[2]
}

/// Length of a 3-vector.
#[inline]
#[must_use]
pub fn vec3_length(v: &[f32; 3]) -> f32 {
    vec3_length_sq(v).sqrt()
}

/// Add two 3-vectors.
#[inline]
#[must_use]
pub fn vec3_add(a: &[f32; 3], b: &[f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

/// Subtract two 3-vectors: `a - b`.
#[inline]
#[must_use]
pub fn vec3_sub(a: &[f32; 3], b: &[f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

/// Scale a 3-vector by a scalar.
#[inline]
#[must_use]
pub fn vec3_scale(v: &[f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

/// Negate a 3-vector.
#[inline]
#[must_use]
pub fn vec3_neg(v: &[f32; 3]) -> [f32; 3] {
    [-v[0], -v[1], -v[2]]
}

/// Build a 3×3 skew-symmetric (cross-product) matrix from a vector.
///
/// `skew(v) * u = v × u` for any u.
#[must_use]
pub fn skew_symmetric(v: &[f32; 3]) -> [f32; 9] {
    [0.0, -v[2], v[1], v[2], 0.0, -v[0], -v[1], v[0], 0.0]
}

/// Compute the generalized inverse mass for a rigid body at an attachment point.
///
/// `w = 1/m + (r × n)ᵀ · I⁻¹ · (r × n)`
///
/// where `r` is the world-space offset from center of mass to the contact point,
/// `n` is the constraint direction, and `I⁻¹` is the world-space inverse inertia.
#[must_use]
pub fn generalized_inverse_mass(
    inv_mass: f32,
    inv_inertia_world: &[f32; 9],
    r: &[f32; 3],
    n: &[f32; 3],
) -> f32 {
    let rxn = vec3_cross(r, n);
    let i_rxn = mat3_mul_vec(inv_inertia_world, &rxn);
    inv_mass + vec3_dot(&rxn, &i_rxn)
}
