//! Computer-graphics recipes: homogeneous 4×4 transforms, MVP, projections, unproject.
//!
//! Inspired by nalgebra's CG support. All matrices are column-major; use [`Matrix::as_slice`]
//! for shader uploads.

use super::math3d::{
    Matrix3f, Matrix4f, Vector3f, Vector4f, make_rotation, matrix4f_inverse, vector3_cross,
};
use crate::quaternion::Quat4f;
use crate::types::Storage;

// --- Homogeneous 4×4 construction (column-major) ---

/// Uniform scaling matrix with scaling factor `s`.
#[must_use]
pub fn new_scaling(s: f32) -> Matrix4f {
    let mut m = Matrix4f::with_storage(4, 4, Storage::Column);
    m.set_zero();
    m.set(0, 0, s);
    m.set(1, 1, s);
    m.set(2, 2, s);
    m.set(3, 3, 1.0);
    m
}

/// Non-uniform scaling matrix; components of `v` are scale factors per axis.
#[must_use]
pub fn new_nonuniform_scaling(v: &Vector3f) -> Matrix4f {
    assert!(v.rows() >= 3);
    let mut m = Matrix4f::with_storage(4, 4, Storage::Column);
    m.set_zero();
    m.set(0, 0, v.get(0));
    m.set(1, 1, v.get(1));
    m.set(2, 2, v.get(2));
    m.set(3, 3, 1.0);
    m
}

/// Pure translation matrix; displacement given by `t`.
#[must_use]
pub fn new_translation(t: &Vector3f) -> Matrix4f {
    assert!(t.rows() >= 3);
    let mut m = Matrix4f::with_storage(4, 4, Storage::Column);
    m.set_zero();
    m.set(0, 0, 1.0);
    m.set(1, 1, 1.0);
    m.set(2, 2, 1.0);
    m.set(3, 3, 1.0);
    m.set(0, 3, t.get(0));
    m.set(1, 3, t.get(1));
    m.set(2, 3, t.get(2));
    m
}

/// Pure rotation matrix from scaled axis: `axis_angle = axis * angle` (angle in radians).
/// Rodrigues formula; identity when angle is zero.
#[must_use]
pub fn from_scaled_axis(axis_angle: &Vector3f) -> Matrix4f {
    assert!(axis_angle.rows() >= 3);
    let x = axis_angle.get(0);
    let y = axis_angle.get(1);
    let z = axis_angle.get(2);
    let angle_sq = x * x + y * y + z * z;
    let mut m = Matrix4f::with_storage(4, 4, Storage::Column);
    m.set_zero();
    m.set(3, 3, 1.0);
    if angle_sq < 1e-14 {
        m.set(0, 0, 1.0);
        m.set(1, 1, 1.0);
        m.set(2, 2, 1.0);
        return m;
    }
    let angle = angle_sq.sqrt();
    let kx = x / angle;
    let ky = y / angle;
    let kz = z / angle;
    let c = angle.cos();
    let s = angle.sin();
    let one_c = 1.0 - c;
    // R = c*I + s*K + (1-c)*k⊗k, K = cross-product matrix of k
    m.set(0, 0, c + one_c * kx * kx);
    m.set(0, 1, -s * kz + one_c * kx * ky);
    m.set(0, 2, s * ky + one_c * kx * kz);
    m.set(1, 0, s * kz + one_c * ky * kx);
    m.set(1, 1, c + one_c * ky * ky);
    m.set(1, 2, -s * kx + one_c * ky * kz);
    m.set(2, 0, -s * ky + one_c * kz * kx);
    m.set(2, 1, s * kx + one_c * kz * ky);
    m.set(2, 2, c + one_c * kz * kz);
    m
}

/// Rotation about point `pt` (point stays fixed). Rotation given by scaled axis `axis_angle`.
#[must_use]
pub fn new_rotation_wrt_point(axis_angle: &Vector3f, pt: &Vector3f) -> Matrix4f {
    let r = from_scaled_axis(axis_angle);
    let t_neg = vector3(-pt.get(0), -pt.get(1), -pt.get(2));
    let t_pos = new_translation(pt);
    let t_neg_m = new_translation(&t_neg);
    // M = T(pt) * R * T(-pt)
    &(&t_pos * &r) * &t_neg_m
}

/// Pure rotation from Euler angles (roll, pitch, yaw) in radians; order roll then pitch then yaw.
#[must_use]
pub fn from_euler_angles(roll: f32, pitch: f32, yaw: f32) -> Matrix4f {
    let r3 = make_rotation(roll, pitch, yaw);
    let mut m = Matrix4f::with_storage(4, 4, Storage::Column);
    m.set_zero();
    for i in 0..3 {
        for j in 0..3 {
            m.set(i, j, r3.get(i, j));
        }
    }
    m.set(3, 3, 1.0);
    m
}

/// Extracts the rotation from a 4×4 transform matrix as a unit quaternion.
///
/// Uses the upper-left 3×3 block; useful for kinematics/render when converting
/// a world transform matrix to a quaternion for display (avoids gimbal lock from Euler).
#[must_use]
pub fn matrix4_extract_rotation_quat(m: &Matrix4f) -> Quat4f {
    let mut r3 = Matrix3f::with_storage(3, 3, Storage::Column);
    for i in 0..3 {
        for j in 0..3 {
            r3.set(i, j, m.get(i, j));
        }
    }
    Quat4f::from_rotation_matrix3(&r3)
}

/// Orthographic projection matrix (column-major). Maps [left,right]×[bottom,top]×[near,far] to NDC.
#[must_use]
pub fn new_orthographic(
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    near: f32,
    far: f32,
) -> Matrix4f {
    let mut m = Matrix4f::with_storage(4, 4, Storage::Column);
    m.set_zero();
    let rml = right - left;
    let tmb = top - bottom;
    let fmn = far - near;
    m.set(0, 0, 2.0 / rml);
    m.set(1, 1, 2.0 / tmb);
    m.set(2, 2, -2.0 / fmn);
    m.set(0, 3, -(right + left) / rml);
    m.set(1, 3, -(top + bottom) / tmb);
    m.set(2, 3, -(far + near) / fmn);
    m.set(3, 3, 1.0);
    m
}

/// Perspective projection matrix (column-major). `aspect = width/height`, `fov_y_rad` vertical FOV in radians, `near` and `far` positive.
/// Maps depth to OpenGL NDC range [-1, 1].
#[must_use]
pub fn new_perspective(aspect: f32, fov_y_rad: f32, near: f32, far: f32) -> Matrix4f {
    let mut m = Matrix4f::with_storage(4, 4, Storage::Column);
    m.set_zero();
    let t = (fov_y_rad * 0.5).tan();
    let sy = 1.0 / t;
    let sx = sy / aspect;
    let nf = near - far;
    m.set(0, 0, sx);
    m.set(1, 1, sy);
    m.set(2, 2, (far + near) / nf);
    m.set(2, 3, (2.0 * far * near) / nf);
    m.set(3, 2, -1.0);
    m
}

/// Perspective projection matrix for WebGPU/Vulkan (column-major). Maps depth to NDC range [0, 1].
/// Use this when rendering with wgpu or WebGPU.
#[must_use]
pub fn new_perspective_wgpu(aspect: f32, fov_y_rad: f32, near: f32, far: f32) -> Matrix4f {
    let mut m = Matrix4f::with_storage(4, 4, Storage::Column);
    m.set_zero();
    let t = (fov_y_rad * 0.5).tan();
    let sy = 1.0 / t;
    let sx = sy / aspect;
    let nf = near - far;
    m.set(0, 0, sx);
    m.set(1, 1, sy);
    m.set(2, 2, far / nf);
    m.set(2, 3, (far * near) / nf);
    m.set(3, 2, -1.0);
    m
}

/// Right-handed look-at view matrix: camera at `eye`, looking at `target`, with `up` as world up.
#[must_use]
pub fn look_at_rh(eye: &Vector3f, target: &Vector3f, up: &Vector3f) -> Matrix4f {
    look_at_impl(eye, target, up, true)
}

/// Left-handed look-at view matrix.
#[must_use]
pub fn look_at_lh(eye: &Vector3f, target: &Vector3f, up: &Vector3f) -> Matrix4f {
    look_at_impl(eye, target, up, false)
}

fn look_at_impl(eye: &Vector3f, target: &Vector3f, up: &Vector3f, right_handed: bool) -> Matrix4f {
    let f = vector3(
        target.get(0) - eye.get(0),
        target.get(1) - eye.get(1),
        target.get(2) - eye.get(2),
    );
    let f = normalize_vec3(&f);
    let s = if right_handed {
        vector3_cross(&f, up)
    } else {
        let neg_f = neg_vec3(&f);
        vector3_cross(&neg_f, up)
    };
    let s = normalize_vec3(&s);
    let u = if right_handed {
        vector3_cross(&s, &f)
    } else {
        let neg_s = neg_vec3(&s);
        vector3_cross(&neg_s, &f)
    };
    let mut m = Matrix4f::with_storage(4, 4, Storage::Column);
    m.set_zero();
    m.set(0, 0, s.get(0));
    m.set(1, 0, s.get(1));
    m.set(2, 0, s.get(2));
    m.set(0, 1, u.get(0));
    m.set(1, 1, u.get(1));
    m.set(2, 1, u.get(2));
    m.set(0, 2, -f.get(0));
    m.set(1, 2, -f.get(1));
    m.set(2, 2, -f.get(2));
    m.set(0, 3, eye.get(0));
    m.set(1, 3, eye.get(1));
    m.set(2, 3, eye.get(2));
    m.set(3, 3, 1.0);
    // Look-at is view matrix: we want world -> camera, so inverse of (R|t). For orthonormal R, inv = R^T and -R^T*t.
    matrix4f_inverse(&m)
}

// --- Append / prepend (out-of-place and in-place) ---

/// Appends uniform scaling: `self * new_scaling(s)`.
#[must_use]
pub fn append_scaling(m: &Matrix4f, s: f32) -> Matrix4f {
    m * &new_scaling(s)
}

/// Prepends uniform scaling: `new_scaling(s) * self`.
#[must_use]
pub fn prepend_scaling(m: &Matrix4f, s: f32) -> Matrix4f {
    &new_scaling(s) * m
}

/// Appends non-uniform scaling: `self * new_nonuniform_scaling(v)`.
#[must_use]
pub fn append_nonuniform_scaling(m: &Matrix4f, v: &Vector3f) -> Matrix4f {
    m * &new_nonuniform_scaling(v)
}

/// Prepends non-uniform scaling: `new_nonuniform_scaling(v) * self`.
#[must_use]
pub fn prepend_nonuniform_scaling(m: &Matrix4f, v: &Vector3f) -> Matrix4f {
    &new_nonuniform_scaling(v) * m
}

/// Appends translation: `self * new_translation(t)`.
#[must_use]
pub fn append_translation(m: &Matrix4f, t: &Vector3f) -> Matrix4f {
    m * &new_translation(t)
}

/// Prepends translation: `new_translation(t) * self`.
#[must_use]
pub fn prepend_translation(m: &Matrix4f, t: &Vector3f) -> Matrix4f {
    &new_translation(t) * m
}

/// In-place: `self = self * new_scaling(s)`.
pub fn append_scaling_mut(m: &mut Matrix4f, s: f32) {
    let s_m = new_scaling(s);
    let out = &*m * &s_m;
    m.copy_from(&out);
}

/// In-place: `self = new_scaling(s) * self`.
pub fn prepend_scaling_mut(m: &mut Matrix4f, s: f32) {
    let s_m = new_scaling(s);
    let out = &s_m * &*m;
    m.copy_from(&out);
}

/// In-place: `self = self * new_nonuniform_scaling(v)`.
pub fn append_nonuniform_scaling_mut(m: &mut Matrix4f, v: &Vector3f) {
    let s_m = new_nonuniform_scaling(v);
    let out = &*m * &s_m;
    m.copy_from(&out);
}

/// In-place: `self = new_nonuniform_scaling(v) * self`.
pub fn prepend_nonuniform_scaling_mut(m: &mut Matrix4f, v: &Vector3f) {
    let s_m = new_nonuniform_scaling(v);
    let out = &s_m * &*m;
    m.copy_from(&out);
}

/// In-place: `self = self * new_translation(t)`.
pub fn append_translation_mut(m: &mut Matrix4f, t: &Vector3f) {
    let t_m = new_translation(t);
    let out = &*m * &t_m;
    m.copy_from(&out);
}

/// In-place: `self = new_translation(t) * self`.
pub fn prepend_translation_mut(m: &mut Matrix4f, t: &Vector3f) {
    let t_m = new_translation(t);
    let out = &t_m * &*m;
    m.copy_from(&out);
}

// --- Transform points and vectors ---

/// Transforms a 3D point (full 4×4, then divide by w for perspective). For affine matrices equivalent to `matrix4_mul_vector3`.
#[must_use]
pub fn transform_point(m: &Matrix4f, p: &Vector3f) -> Vector3f {
    let h = vector4_from_point(p.get(0), p.get(1), p.get(2));
    let out4 = m * &h;
    from_homogeneous(&out4).unwrap_or_else(|| vector3(out4.get(0), out4.get(1), out4.get(2)))
}

/// Extracts the translation vector from a 4×4 affine matrix (column 3, rows 0–2).
#[must_use]
pub fn matrix4f_translation(m: &Matrix4f) -> Vector3f {
    vector3(m.get(0, 3), m.get(1, 3), m.get(2, 3))
}

// --- Homogeneous helpers ---

/// Build a 3D vector (x, y, z).
#[must_use]
pub fn vector3(x: f32, y: f32, z: f32) -> Vector3f {
    let mut v = Vector3f::with_capacity(3);
    v.set(0, x);
    v.set(1, y);
    v.set(2, z);
    v
}

/// Homogeneous point (x, y, z, 1).
#[must_use]
pub fn vector4_from_point(x: f32, y: f32, z: f32) -> Vector4f {
    let mut v = Vector4f::with_capacity(4);
    v.set(0, x);
    v.set(1, y);
    v.set(2, z);
    v.set(3, 1.0);
    v
}

/// Homogeneous direction (x, y, z, 0).
#[must_use]
pub fn vector4_from_vector(x: f32, y: f32, z: f32) -> Vector4f {
    let mut v = Vector4f::with_capacity(4);
    v.set(0, x);
    v.set(1, y);
    v.set(2, z);
    v.set(3, 0.0);
    v
}

/// Recover 3D point from homogeneous; returns `None` if w ≈ 0.
#[must_use]
pub fn from_homogeneous(v: &Vector4f) -> Option<Vector3f> {
    assert!(v.rows() >= 4);
    let w = v.get(3);
    if w.abs() < 1e-9 {
        return None;
    }
    Some(vector3(v.get(0) / w, v.get(1) / w, v.get(2) / w))
}

// --- MVP ---

/// Model-View-Projection matrix: `projection * view * model` (column-major).
#[must_use]
pub fn model_view_projection(model: &Matrix4f, view: &Matrix4f, projection: &Matrix4f) -> Matrix4f {
    let mv = view * model;
    projection * &mv
}

/// Copy 4×4 matrix to array (column-major) for shader uniform upload.
#[must_use]
pub fn matrix4f_to_array(m: &Matrix4f) -> [f32; 16] {
    let s = m.data();
    let mut arr = [0.0f32; 16];
    arr.copy_from_slice(s);
    arr
}

/// 4×4 identity matrix (column-major).
#[must_use]
pub fn matrix4f_identity() -> Matrix4f {
    let mut m = Matrix4f::with_storage(4, 4, Storage::Column);
    m.set_identity();
    m
}

// --- Perspective3 and unproject ---

/// Perspective projection parameters; use [`Perspective3::as_matrix`] and [`Perspective3::unproject_point`].
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Perspective3 {
    pub(crate) aspect: f32,
    pub(crate) fov_y_rad: f32,
    pub(crate) near: f32,
    pub(crate) far: f32,
}

impl Perspective3 {
    /// `aspect = width/height`, `fov_y_rad` vertical FOV in radians, `near` and `far` positive.
    #[must_use]
    pub fn new(aspect: f32, fov_y_rad: f32, near: f32, far: f32) -> Self {
        Self {
            aspect,
            fov_y_rad,
            near,
            far,
        }
    }

    /// 4×4 perspective matrix (column-major).
    #[must_use]
    pub fn as_matrix(&self) -> Matrix4f {
        new_perspective(self.aspect, self.fov_y_rad, self.near, self.far)
    }

    /// Inverse of the perspective matrix (for unproject).
    #[must_use]
    pub fn inverse_matrix(&self) -> Matrix4f {
        matrix4f_inverse(&self.as_matrix())
    }

    /// Unproject a point from NDC (normalized device coords, z in [-1,1]) to view space.
    #[must_use]
    pub fn unproject_point(&self, ndc: &Vector3f) -> Vector3f {
        let inv = self.inverse_matrix();
        let h = vector4_from_point(ndc.get(0), ndc.get(1), ndc.get(2));
        let v = &inv * &h;
        from_homogeneous(&v).unwrap_or_else(|| vector3(v.get(0), v.get(1), v.get(2)))
    }
}

/// Screen-space (pixel coords) to view-space ray. Returns (origin, direction) in view space; direction is normalized.
#[must_use]
pub fn screen_to_view_ray(
    projection: &Perspective3,
    screen_x: f32,
    screen_y: f32,
    screen_width: f32,
    screen_height: f32,
) -> (Vector3f, Vector3f) {
    // Screen to NDC: NDC x in [-1,1], y in [-1,1]. Typical: ndc_x = 2*sx/width - 1, ndc_y = 1 - 2*sy/height.
    let ndc_x = 2.0 * screen_x / screen_width - 1.0;
    let ndc_y = 1.0 - 2.0 * screen_y / screen_height;
    let near_ndc = vector3(ndc_x, ndc_y, -1.0);
    let far_ndc = vector3(ndc_x, ndc_y, 1.0);
    let near_view = projection.unproject_point(&near_ndc);
    let far_view = projection.unproject_point(&far_ndc);
    let dir = vector3(
        far_view.get(0) - near_view.get(0),
        far_view.get(1) - near_view.get(1),
        far_view.get(2) - near_view.get(2),
    );
    let direction = normalize_vec3(&dir);
    (near_view, direction)
}

// --- Internal helpers ---

fn neg_vec3(v: &Vector3f) -> Vector3f {
    vector3(-v.get(0), -v.get(1), -v.get(2))
}

fn normalize_vec3(v: &Vector3f) -> Vector3f {
    v.normalize()
}
