//! Ray casting: ray–AABB, ray–triangle (Möller–Trumbore), ray–segment, ray–sphere, ray–OBB, ray–capsule.

use crate::aabb::ray_aabb as aabb_ray_aabb;
use crate::{Obb, Sphere};

/// Re-export of [`crate::aabb::ray_aabb`] for API convenience.
#[must_use]
pub fn ray_aabb(origin: &[f32; 3], dir: &[f32; 3], aabb: &crate::Aabb) -> Option<f32> {
    aabb_ray_aabb(origin, dir, aabb)
}

/// Möller–Trumbore ray–triangle intersection.
///
/// Returns `Some(t)` where `origin + t * dir` hits the triangle (v0, v1, v2), or `None`.
#[must_use]
pub fn ray_triangle(
    origin: &[f32; 3],
    dir: &[f32; 3],
    v0: &[f32; 3],
    v1: &[f32; 3],
    v2: &[f32; 3],
) -> Option<f32> {
    let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
    let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
    let h = [
        dir[1] * e2[2] - dir[2] * e2[1],
        dir[2] * e2[0] - dir[0] * e2[2],
        dir[0] * e2[1] - dir[1] * e2[0],
    ];
    let a = e1[0] * h[0] + e1[1] * h[1] + e1[2] * h[2];
    let eps = 1e-7;
    if a > -eps && a < eps {
        return None;
    }
    let f = 1.0 / a;
    let s = [origin[0] - v0[0], origin[1] - v0[1], origin[2] - v0[2]];
    let u = f * (s[0] * h[0] + s[1] * h[1] + s[2] * h[2]);
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let q = [
        s[1] * e1[2] - s[2] * e1[1],
        s[2] * e1[0] - s[0] * e1[2],
        s[0] * e1[1] - s[1] * e1[0],
    ];
    let v = f * (dir[0] * q[0] + dir[1] * q[1] + dir[2] * q[2]);
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    let t = f * (e2[0] * q[0] + e2[1] * q[1] + e2[2] * q[2]);
    if t > eps {
        Some(t)
    } else {
        None
    }
}

fn ray_segment_parallel(
    _origin: &[f32; 3],
    dir: &[f32; 3],
    _a: &[f32; 3],
    _b: &[f32; 3],
    w: &[f32; 3],
    v: &[f32; 3],
    v_len: f32,
) -> Option<f32> {
    let eps = 1e-9_f32;
    let t_eps = 1e-5_f32;
    let cross_x = w[1] * dir[2] - w[2] * dir[1];
    let cross_y = w[2] * dir[0] - w[0] * dir[2];
    let cross_z = w[0] * dir[1] - w[1] * dir[0];
    let cross_len = cross_x.abs() + cross_y.abs() + cross_z.abs();
    if cross_len > eps * (v_len + 1.0) {
        return None;
    }
    let v_dot = v[0] * v[0] + v[1] * v[1] + v[2] * v[2];
    if v_dot < eps {
        return None;
    }
    let u0 = (w[0] * v[0] + w[1] * v[1] + w[2] * v[2]) / v_dot;
    let d_dot_v = dir[0] * v[0] + dir[1] * v[1] + dir[2] * v[2];
    let c = if d_dot_v.abs() < eps {
        return None;
    } else {
        d_dot_v / v_dot
    };
    if (-1e-5..=1.0 + 1e-5).contains(&u0) {
        return Some(t_eps);
    }
    if c > 0.0 && u0 < 0.0 {
        let t = -u0 / c;
        if t >= t_eps {
            return Some(t);
        }
    }
    if c < 0.0 && u0 > 1.0 {
        let t = (1.0 - u0) / c;
        if t >= t_eps {
            return Some(t);
        }
    }
    None
}

/// Ray–line-segment intersection.
///
/// Returns the smallest non-negative `t` such that `origin + t * dir` lies on the segment
/// between `a` and `b`, or `None` if there is no such hit.
#[must_use]
pub fn ray_segment(origin: &[f32; 3], dir: &[f32; 3], a: &[f32; 3], b: &[f32; 3]) -> Option<f32> {
    let v = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let w = [origin[0] - a[0], origin[1] - a[1], origin[2] - a[2]];
    let eps = 1e-9;
    let t_eps = 1e-5;

    let v_len = v[0].abs() + v[1].abs() + v[2].abs();
    if v_len < eps {
        return None;
    }

    let (t, u) = {
        let denom_xy = dir[0] * v[1] - dir[1] * v[0];
        let denom_xz = dir[0] * v[2] - dir[2] * v[0];
        let denom_yz = dir[1] * v[2] - dir[2] * v[1];
        if denom_xy.abs() >= denom_xz.abs()
            && denom_xy.abs() >= denom_yz.abs()
            && denom_xy.abs() >= eps
        {
            let t = (v[0] * w[1] - w[0] * v[1]) / denom_xy;
            let u = (w[0] + t * dir[0]) / v[0];
            (t, u)
        } else if denom_xz.abs() >= denom_xy.abs()
            && denom_xz.abs() >= denom_yz.abs()
            && denom_xz.abs() >= eps
        {
            let t = (v[0] * w[2] - w[0] * v[2]) / denom_xz;
            let u = (w[0] + t * dir[0]) / v[0];
            (t, u)
        } else if denom_yz.abs() >= eps {
            let t = (v[1] * w[2] - w[1] * v[2]) / denom_yz;
            let u = (w[1] + t * dir[1]) / v[1];
            (t, u)
        } else {
            return ray_segment_parallel(origin, dir, a, b, &w, &v, v_len);
        }
    };

    if t < -t_eps || !(-1e-5..=1.0 + 1e-5).contains(&u) {
        return None;
    }
    let hit_x = origin[0] + t * dir[0];
    let seg_x = a[0] + u * (b[0] - a[0]);
    let hit_y = origin[1] + t * dir[1];
    let seg_y = a[1] + u * (b[1] - a[1]);
    let hit_z = origin[2] + t * dir[2];
    let seg_z = a[2] + u * (b[2] - a[2]);
    let err = (hit_x - seg_x).abs() + (hit_y - seg_y).abs() + (hit_z - seg_z).abs();
    if err < 1e-3 && t >= t_eps {
        Some(t)
    } else {
        None
    }
}

/// Ray–sphere intersection. Returns the smallest non-negative `t` or `None`.
#[must_use]
pub fn ray_sphere(origin: &[f32; 3], dir: &[f32; 3], sphere: &Sphere) -> Option<f32> {
    sphere.ray_intersect(origin, dir)
}

/// Ray–OBB intersection. Returns the smallest non-negative `t` or `None`.
#[must_use]
pub fn ray_obb(origin: &[f32; 3], dir: &[f32; 3], obb: &Obb) -> Option<f32> {
    obb.ray_intersect(origin, dir)
}

#[cfg(test)]
mod tests {
    use super::{ray_aabb, ray_obb, ray_segment, ray_sphere, ray_triangle};
    use crate::ray_capsule;
    use crate::{Aabb, Capsule, Obb, Sphere};

    #[test]
    fn ray_aabb_hit() {
        let aabb = Aabb::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let origin = [0.5, 0.5, 2.0];
        let dir = [0.0, 0.0, -1.0];
        let t = ray_aabb(&origin, &dir, &aabb).unwrap();
        assert!((t - 1.0).abs() < 1e-5);
    }

    #[test]
    fn ray_aabb_miss() {
        let aabb = Aabb::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let origin = [2.0, 2.0, 2.0];
        let dir = [1.0, 0.0, 0.0];
        assert!(ray_aabb(&origin, &dir, &aabb).is_none());
    }

    #[test]
    fn ray_triangle_hit() {
        let origin = [0.0, 0.0, 1.0];
        let dir = [0.0, 0.0, -1.0];
        let v0 = [0.0, 0.0, 0.0];
        let v1 = [1.0, 0.0, 0.0];
        let v2 = [0.5, 1.0, 0.0];
        let t = ray_triangle(&origin, &dir, &v0, &v1, &v2).unwrap();
        assert!(t > 0.0 && t < 2.0);
    }

    #[test]
    fn ray_triangle_miss_parallel() {
        // Ray parallel to triangle plane (in plane normal direction but no intersection)
        let v0 = [0.0, 0.0, 0.0];
        let v1 = [1.0, 0.0, 0.0];
        let v2 = [0.5, 1.0, 0.0];
        let origin = [0.5, 0.5, 1.0];
        let dir = [1.0, 0.0, 0.0]; // parallel to plane
        assert!(ray_triangle(&origin, &dir, &v0, &v1, &v2).is_none());
    }

    #[test]
    fn ray_triangle_miss_away() {
        let v0 = [0.0, 0.0, 0.0];
        let v1 = [1.0, 0.0, 0.0];
        let v2 = [0.5, 1.0, 0.0];
        let origin = [0.5, 0.5, 1.0];
        let dir = [0.0, 0.0, 1.0]; // away from triangle
        assert!(ray_triangle(&origin, &dir, &v0, &v1, &v2).is_none());
    }

    #[test]
    fn ray_triangle_miss_outside_uv() {
        // Ray hits plane but outside triangle (u or v out of [0,1] or u+v > 1)
        let v0 = [0.0, 0.0, 0.0];
        let v1 = [1.0, 0.0, 0.0];
        let v2 = [0.5, 1.0, 0.0];
        let origin = [2.0, 2.0, 1.0];
        let dir = [0.0, 0.0, -1.0];
        assert!(ray_triangle(&origin, &dir, &v0, &v1, &v2).is_none());
    }

    #[test]
    fn ray_segment_hit() {
        let origin = [0.5, 0.5, 1.0];
        let dir = [0.0, 0.0, -1.0];
        let a = [0.0, 0.0, 0.0];
        let b = [1.0, 1.0, 0.0];
        let t = ray_segment(&origin, &dir, &a, &b).unwrap();
        assert!(t > 0.0);
    }

    #[test]
    fn ray_segment_miss() {
        let origin = [0.0, 0.0, 2.0];
        let dir = [0.0, 0.0, -1.0];
        let a = [1.0, 1.0, 0.0];
        let b = [2.0, 2.0, 0.0];
        assert!(ray_segment(&origin, &dir, &a, &b).is_none());
    }

    #[test]
    fn ray_segment_miss_parallel() {
        let origin = [0.0, 0.0, 1.0];
        let dir = [1.0, 0.0, 0.0];
        let a = [0.0, 1.0, 0.0];
        let b = [1.0, 1.0, 0.0];
        assert!(ray_segment(&origin, &dir, &a, &b).is_none());
    }

    #[test]
    fn ray_obb_hit() {
        let rot = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
        let obb = Obb::new([0.0, 0.0, 0.0], [0.5, 0.5, 0.5], rot);
        let origin = [0.0, 0.0, 2.0];
        let dir = [0.0, 0.0, -1.0];
        let t = ray_obb(&origin, &dir, &obb).unwrap();
        assert!(t > 0.0 && t < 3.0);
    }

    #[test]
    fn ray_obb_miss() {
        let rot = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
        let obb = Obb::new([0.0, 0.0, 0.0], [0.5, 0.5, 0.5], rot);
        let origin = [0.0, 0.0, 2.0];
        let dir = [0.0, 0.0, 1.0];
        assert!(ray_obb(&origin, &dir, &obb).is_none());
    }

    #[test]
    fn ray_sphere_miss() {
        let s = Sphere::new([0.0, 0.0, 0.0], 1.0);
        let origin = [0.0, 0.0, 2.0];
        let dir = [0.0, 0.0, 1.0];
        assert!(ray_sphere(&origin, &dir, &s).is_none());
    }

    #[test]
    fn ray_capsule_hit() {
        let cap = Capsule::new([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], 0.5);
        let origin = [0.5, 0.0, 2.0];
        let dir = [0.0, 0.0, -1.0];
        let t = ray_capsule(&origin, &dir, &cap).unwrap();
        assert!(t > 0.0 && t < 3.0);
    }

    #[test]
    fn ray_capsule_miss() {
        let cap = Capsule::new([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], 0.1);
        let origin = [5.0, 5.0, 1.0];
        let dir = [0.0, 0.0, -1.0];
        assert!(ray_capsule(&origin, &dir, &cap).is_none());
    }
}
