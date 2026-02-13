//! Frustum and AABB culling for visibility.
//!
//! Re-exports [`Aabb`], [`Frustum`], and [`ray_aabb`] from the **collision** crate.
//! Provides model-space AABBs for primitives and world AABB transform (scene-specific).

pub use collision::{ray_aabb, Aabb, Frustum};

use mathlib::cg::{transform_point, vector3};
use mathlib::math3d::Matrix4f;

use crate::scene::{Primitive, Primitive2D, Primitive3D};

/// Returns a conservative model-space AABB for the primitive.
/// Matches the conventions used in [`crate::backend::primitive_mesh`].
#[must_use]
pub fn primitive_aabb(prim: &Primitive) -> Aabb {
    match prim {
        Primitive::TwoD(p) => primitive_2d_aabb(p),
        Primitive::ThreeD(p) => primitive_3d_aabb(p),
    }
}

fn primitive_2d_aabb(p: &Primitive2D) -> Aabb {
    match p {
        Primitive2D::Quad => Aabb::new([-1.0, -1.0, 0.0], [1.0, 1.0, 0.0]),
        Primitive2D::Square => Aabb::new([-0.5, -0.5, 0.0], [0.5, 0.5, 0.0]),
        Primitive2D::Circle => Aabb::new([-1.0, -1.0, 0.0], [1.0, 1.0, 0.0]),
        Primitive2D::Ellipse => Aabb::new([-0.5, -0.25, 0.0], [0.5, 0.25, 0.0]),
        Primitive2D::Triangle => Aabb::new([-0.5, -0.5, 0.0], [0.5, 0.5, 0.0]),
    }
}

fn primitive_3d_aabb(p: &Primitive3D) -> Aabb {
    match p {
        Primitive3D::Quad => Aabb::new([-0.5, -0.5, 0.0], [0.5, 0.5, 0.0]),
        Primitive3D::Triangle => Aabb::new([-0.5, -0.5, 0.0], [0.5, 0.5, 0.0]),
        Primitive3D::Cube => Aabb::new([-0.5, -0.5, -0.5], [0.5, 0.5, 0.5]),
        Primitive3D::Tetrahedron => {
            let s = 0.5_f32;
            let t = 1.0_f32 / (2.0_f32).sqrt();
            let h = s * (2.0_f32).sqrt();
            Aabb::new([-s * t, -s, 0.0], [s * t, s, h])
        }
        Primitive3D::Cylinder => Aabb::new([-0.5, -0.5, -0.5], [0.5, 0.5, 0.5]),
        Primitive3D::Sphere => Aabb::new([-0.5, -0.5, -0.5], [0.5, 0.5, 0.5]),
        Primitive3D::Cone => Aabb::new([-0.5, -0.5, -0.5], [0.5, 0.5, 0.5]),
        Primitive3D::Capsule => Aabb::new([-0.5, -0.5, -0.9], [0.5, 0.5, 0.9]),
        Primitive3D::LineSegment { start, end } => {
            let s = start.0;
            let e = end.0;
            let min = [s[0].min(e[0]), s[1].min(e[1]), s[2].min(e[2])];
            let max = [s[0].max(e[0]), s[1].max(e[1]), s[2].max(e[2])];
            Aabb::new(min, max)
        }
        Primitive3D::Bezier { control_points } => {
            let mut min = [f32::INFINITY; 3];
            let mut max = [f32::NEG_INFINITY; 3];
            for cp in control_points {
                for (i, &v) in cp.0.iter().enumerate() {
                    min[i] = min[i].min(v);
                    max[i] = max[i].max(v);
                }
            }
            Aabb::new(min, max)
        }
        Primitive3D::Hermite { p0, p1, m0, m1 } => {
            let mut min = [f32::INFINITY; 3];
            let mut max = [f32::NEG_INFINITY; 3];
            for p in [p0.0, p1.0, m0.0, m1.0] {
                for (i, &v) in p.iter().enumerate() {
                    min[i] = min[i].min(v);
                    max[i] = max[i].max(v);
                }
            }
            Aabb::new(min, max)
        }
        Primitive3D::BSpline { control_points } => {
            let mut min = [f32::INFINITY; 3];
            let mut max = [f32::NEG_INFINITY; 3];
            for cp in control_points {
                for (i, &v) in cp.0.iter().enumerate() {
                    min[i] = min[i].min(v);
                    max[i] = max[i].max(v);
                }
            }
            Aabb::new(min, max)
        }
    }
}

/// Transforms an AABB from model space to world space by transforming
/// all eight corners (affine) and taking the min/max.
#[must_use]
pub fn world_aabb(model_aabb: &Aabb, world: &Matrix4f) -> Aabb {
    let corners = model_aabb.corners();
    let mut min = [f32::INFINITY; 3];
    let mut max = [f32::NEG_INFINITY; 3];
    for corner in corners {
        let [x, y, z] = corner;
        let p = vector3(x, y, z);
        let out = transform_point(world, &p);
        let wx = out.get(0);
        let wy = out.get(1);
        let wz = out.get(2);
        min[0] = min[0].min(wx);
        min[1] = min[1].min(wy);
        min[2] = min[2].min(wz);
        max[0] = max[0].max(wx);
        max[1] = max[1].max(wy);
        max[2] = max[2].max(wz);
    }
    Aabb::new(min, max)
}
