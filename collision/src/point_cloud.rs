//! Point-cloud AABB and bounding sphere helpers.
//!
//! Uses mathlib for transform (point_cloud_aabb) and optional SIMD for min/max reduction.

use mathlib::cg::{transform_point, vector3};
use mathlib::math3d::Matrix4f;

use crate::Aabb;

/// Computes the AABB of a set of points transformed by `m`.
///
/// # Panics
///
/// Panics if the iterator yields no points.
#[must_use]
pub fn point_cloud_aabb<I>(m: &Matrix4f, pts: I) -> Aabb
where
    I: IntoIterator<Item = [f32; 3]>,
{
    let mut it = pts.into_iter();
    let p0 = it
        .next()
        .expect("point_cloud_aabb: the input iterator must yield at least one point");
    let v0 = vector3(p0[0], p0[1], p0[2]);
    let w0 = transform_point(m, &v0);
    let mut min = [w0.get(0), w0.get(1), w0.get(2)];
    let mut max = [w0.get(0), w0.get(1), w0.get(2)];

    #[cfg(feature = "simd")]
    {
        use wide::f32x4;
        let mut min_v = f32x4::from([min[0], min[1], min[2], 0.0]);
        let mut max_v = f32x4::from([max[0], max[1], max[2], 0.0]);
        for pt in it {
            let v = vector3(pt[0], pt[1], pt[2]);
            let w = transform_point(m, &v);
            let wv = f32x4::from([w.get(0), w.get(1), w.get(2), 0.0]);
            min_v = min_v.fast_min(wv);
            max_v = max_v.fast_max(wv);
        }
        let min_arr: [f32; 4] = min_v.to_array();
        let max_arr: [f32; 4] = max_v.to_array();
        min = [min_arr[0], min_arr[1], min_arr[2]];
        max = [max_arr[0], max_arr[1], max_arr[2]];
    }

    #[cfg(not(feature = "simd"))]
    {
        for pt in it {
            let v = vector3(pt[0], pt[1], pt[2]);
            let w = transform_point(m, &v);
            let wx = w.get(0);
            let wy = w.get(1);
            let wz = w.get(2);
            min[0] = min[0].min(wx);
            min[1] = min[1].min(wy);
            min[2] = min[2].min(wz);
            max[0] = max[0].max(wx);
            max[1] = max[1].max(wy);
            max[2] = max[2].max(wz);
        }
    }

    Aabb::new(min, max)
}

/// Transforms an AABB from model space to world space by transforming
/// all eight corners and taking the min/max.
#[must_use]
pub fn world_aabb(model_aabb: &Aabb, world: &Matrix4f) -> Aabb {
    let corners = model_aabb.corners();
    let mut pts = Vec::with_capacity(8);
    for corner in corners {
        let [x, y, z] = corner;
        let p = vector3(x, y, z);
        let out = transform_point(world, &p);
        pts.push([out.get(0), out.get(1), out.get(2)]);
    }
    local_point_cloud_aabb(pts)
}

/// Computes the AABB of a set of point references transformed by `m`.
///
/// # Panics
///
/// Panics if the iterator yields no points.
#[must_use]
pub fn point_cloud_aabb_ref<'a, I>(m: &Matrix4f, pts: I) -> Aabb
where
    I: IntoIterator<Item = &'a [f32; 3]>,
{
    point_cloud_aabb(m, pts.into_iter().copied())
}

/// Computes the local-space AABB of a set of points.
///
/// # Panics
///
/// Panics if the iterator yields no points.
#[must_use]
pub fn local_point_cloud_aabb<I>(pts: I) -> Aabb
where
    I: IntoIterator<Item = [f32; 3]>,
{
    let mut it = pts.into_iter();
    let p0 = it
        .next()
        .expect("local_point_cloud_aabb: the input iterator must yield at least one point");
    let mut min = p0;
    let mut max = p0;

    #[cfg(feature = "simd")]
    {
        use wide::f32x4;
        let mut min_v = f32x4::from([min[0], min[1], min[2], 0.0]);
        let mut max_v = f32x4::from([max[0], max[1], max[2], 0.0]);
        for pt in it {
            let pv = f32x4::from([pt[0], pt[1], pt[2], 0.0]);
            min_v = min_v.fast_min(pv);
            max_v = max_v.fast_max(pv);
        }
        let min_arr: [f32; 4] = min_v.to_array();
        let max_arr: [f32; 4] = max_v.to_array();
        min = [min_arr[0], min_arr[1], min_arr[2]];
        max = [max_arr[0], max_arr[1], max_arr[2]];
    }

    #[cfg(not(feature = "simd"))]
    {
        for pt in it {
            min[0] = min[0].min(pt[0]);
            min[1] = min[1].min(pt[1]);
            min[2] = min[2].min(pt[2]);
            max[0] = max[0].max(pt[0]);
            max[1] = max[1].max(pt[1]);
            max[2] = max[2].max(pt[2]);
        }
    }

    Aabb::new(min, max)
}

/// Computes the local-space AABB of a set of point references.
///
/// # Panics
///
/// Panics if the iterator yields no points.
#[must_use]
pub fn local_point_cloud_aabb_ref<'a, I>(pts: I) -> Aabb
where
    I: IntoIterator<Item = &'a [f32; 3]>,
{
    local_point_cloud_aabb(pts.into_iter().copied())
}

/// Conservative bounding sphere for a set of points: center is centroid, radius is
/// maximum distance from center to any point.
///
/// # Panics
///
/// Panics if the slice is empty.
#[must_use]
pub fn point_cloud_bounding_sphere(pts: &[[f32; 3]]) -> ([f32; 3], f32) {
    assert!(
        !pts.is_empty(),
        "point_cloud_bounding_sphere: pts must not be empty"
    );
    let n = pts.len() as f32;
    let mut cx = 0.0;
    let mut cy = 0.0;
    let mut cz = 0.0;
    for p in pts {
        cx += p[0];
        cy += p[1];
        cz += p[2];
    }
    cx /= n;
    cy /= n;
    cz /= n;
    let center = [cx, cy, cz];
    let mut max_dist_sq = 0.0f32;
    for p in pts {
        let dx = p[0] - cx;
        let dy = p[1] - cy;
        let dz = p[2] - cz;
        let d_sq = dx * dx + dy * dy + dz * dz;
        if d_sq > max_dist_sq {
            max_dist_sq = d_sq;
        }
    }
    let radius = max_dist_sq.sqrt();
    (center, radius)
}

#[cfg(test)]
mod tests {
    use super::{
        local_point_cloud_aabb, point_cloud_aabb, point_cloud_aabb_ref,
        point_cloud_bounding_sphere, world_aabb,
    };
    use crate::Aabb;
    use mathlib::cg::matrix4f_identity;

    #[test]
    fn local_point_cloud_aabb_single() {
        let pts = [[1.0, 2.0, 3.0]];
        let aabb = local_point_cloud_aabb(pts);
        assert_eq!(aabb.min, [1.0, 2.0, 3.0]);
        assert_eq!(aabb.max, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn local_point_cloud_aabb_two() {
        let pts = [[0.0, 0.0, 0.0], [1.0, 2.0, 3.0]];
        let aabb = local_point_cloud_aabb(pts);
        assert_eq!(aabb.min, [0.0, 0.0, 0.0]);
        assert_eq!(aabb.max, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn point_cloud_aabb_identity() {
        let pts = [[0.0, 0.0, 0.0], [1.0, 1.0, 1.0]];
        let m = matrix4f_identity();
        let aabb = point_cloud_aabb(&m, pts);
        assert_eq!(aabb.min, [0.0, 0.0, 0.0]);
        assert_eq!(aabb.max, [1.0, 1.0, 1.0]);
    }

    #[test]
    fn point_cloud_aabb_ref_identity() {
        let pts = [[0.0, 0.0, 0.0], [1.0, 1.0, 1.0]];
        let m = matrix4f_identity();
        let aabb = point_cloud_aabb_ref(&m, pts.iter());
        assert_eq!(aabb.min, [0.0, 0.0, 0.0]);
        assert_eq!(aabb.max, [1.0, 1.0, 1.0]);
    }

    #[test]
    fn point_cloud_bounding_sphere_single() {
        let pts = [[1.0, 0.0, 0.0]];
        let (center, radius) = point_cloud_bounding_sphere(&pts);
        assert_eq!(center, [1.0, 0.0, 0.0]);
        assert_eq!(radius, 0.0);
    }

    #[test]
    fn point_cloud_bounding_sphere_two() {
        let pts = [[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let (center, radius) = point_cloud_bounding_sphere(&pts);
        assert_eq!(center, [1.0, 0.0, 0.0]);
        assert!((radius - 1.0).abs() < 1e-5);
    }

    #[test]
    fn world_aabb_identity() {
        let model = Aabb::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let m = matrix4f_identity();
        let world = world_aabb(&model, &m);
        assert_eq!(world.min, model.min);
        assert_eq!(world.max, model.max);
    }
}
