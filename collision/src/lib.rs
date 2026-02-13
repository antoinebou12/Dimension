//! Collision and spatial query crate for Dimension.
//!
//! Provides axis-aligned bounding boxes (3D [`Aabb`], 2D [`Aabb2`]), ray casting
//! (ray–AABB, ray–triangle, ray–segment, ray–sphere, ray–OBB, ray–capsule; 2D
//! [`ray_aabb_2d`], [`ray_circle_2d`]), view frustum extraction and AABB culling,
//! point-cloud AABB and bounding sphere ([`point_cloud_aabb`], [`local_point_cloud_aabb`],
//! [`point_cloud_bounding_sphere`], [`world_aabb`]), and spatial acceleration
//! structures (BSP tree, BVH). Optional shapes: sphere, OBB, capsule; 2D [`Circle`].
//!
//! # Examples
//!
//! ```
//! use collision::{Aabb, ray_aabb, Frustum};
//! use mathlib::cg::matrix4f_identity;
//!
//! let aabb = Aabb::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
//! let origin = [0.5, 0.5, 2.0];
//! let dir = [0.0, 0.0, -1.0];
//! let t = ray_aabb(&origin, &dir, &aabb).unwrap();
//! assert!((t - 1.0).abs() < 1e-5);
//!
//! let view_proj = matrix4f_identity();
//! let frustum = Frustum::from_view_proj(&view_proj);
//! assert!(frustum.intersects_aabb(&aabb));
//! ```

pub mod aabb;
pub mod aabb2;
pub mod bsp;
pub mod bvh;
pub mod capsule;
pub mod circle;
pub mod convex2d;
pub mod frustum;
pub mod obb;
pub mod point_cloud;
pub mod ray;
pub mod sphere;

pub use aabb::Aabb;
pub use aabb2::{ray_aabb_2d, Aabb2};
pub use bsp::{BspNode, BspTree};
pub use bvh::{BvhBuildStrategy, BvhFrustumIter, BvhNode, BvhRayIter, BvhTree};
pub use capsule::ray_capsule;
pub use capsule::Capsule;
pub use circle::{ray_circle_2d, Circle};
pub use convex2d::{convex_hull_2d, point_in_polygon_2d, ray_polygon_2d};
pub use frustum::Frustum;
pub use obb::Obb;
pub use point_cloud::{
    local_point_cloud_aabb, local_point_cloud_aabb_ref, point_cloud_aabb, point_cloud_aabb_ref,
    point_cloud_bounding_sphere, world_aabb,
};
pub use ray::{ray_aabb, ray_obb, ray_segment, ray_sphere, ray_triangle};
pub use sphere::Sphere;
