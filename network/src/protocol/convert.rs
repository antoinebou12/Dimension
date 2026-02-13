//! Conversion between protocol types and mathlib, render, collision, kinematics types.

use crate::protocol::proto::{self, Color, Transform, Vec3};

impl From<[f32; 3]> for Vec3 {
    fn from(v: [f32; 3]) -> Self {
        Vec3 {
            x: v[0],
            y: v[1],
            z: v[2],
        }
    }
}

impl From<Vec3> for [f32; 3] {
    fn from(v: Vec3) -> Self {
        [v.x, v.y, v.z]
    }
}

impl From<[f32; 4]> for Color {
    fn from(c: [f32; 4]) -> Self {
        Color {
            r: c[0],
            g: c[1],
            b: c[2],
            a: c[3],
        }
    }
}

impl From<Color> for [f32; 4] {
    fn from(c: Color) -> Self {
        [c.r, c.g, c.b, c.a]
    }
}

impl From<proto::Transform> for [f32; 9] {
    fn from(t: proto::Transform) -> Self {
        let p = t
            .position
            .as_ref()
            .map(|v| [v.x, v.y, v.z])
            .unwrap_or([0.0; 3]);
        let r = t
            .rotation
            .as_ref()
            .map(|v| [v.x, v.y, v.z])
            .unwrap_or([0.0; 3]);
        let s = t
            .scale
            .as_ref()
            .map(|v| [v.x, v.y, v.z])
            .unwrap_or([1.0; 3]);
        [p[0], p[1], p[2], r[0], r[1], r[2], s[0], s[1], s[2]]
    }
}

/// Build a proto Transform from position, rotation, scale arrays.
#[must_use]
pub fn transform_from_arrays(position: [f32; 3], rotation: [f32; 3], scale: [f32; 3]) -> Transform {
    Transform {
        position: Some(Vec3::from(position)),
        rotation: Some(Vec3::from(rotation)),
        scale: Some(Vec3::from(scale)),
    }
}

#[cfg(feature = "render")]
impl From<render::Transform> for Transform {
    fn from(t: render::Transform) -> Self {
        transform_from_arrays(t.position, t.rotation, t.scale)
    }
}

#[cfg(feature = "render")]
impl From<Transform> for render::Transform {
    fn from(t: Transform) -> Self {
        let p = t
            .position
            .as_ref()
            .map(|v| [v.x, v.y, v.z])
            .unwrap_or([0.0; 3]);
        let r = t
            .rotation
            .as_ref()
            .map(|v| [v.x, v.y, v.z])
            .unwrap_or([0.0; 3]);
        let s = t
            .scale
            .as_ref()
            .map(|v| [v.x, v.y, v.z])
            .unwrap_or([1.0; 3]);
        render::Transform {
            position: p,
            rotation: r,
            rotation_quat: None,
            scale: s,
        }
    }
}

impl From<collision::Aabb> for proto::AabbState {
    fn from(aabb: collision::Aabb) -> Self {
        proto::AabbState {
            min: Some(Vec3::from(aabb.min)),
            max: Some(Vec3::from(aabb.max)),
        }
    }
}

impl TryFrom<proto::AabbState> for collision::Aabb {
    type Error = crate::NetworkError;

    fn try_from(s: proto::AabbState) -> Result<Self, Self::Error> {
        let min = s
            .min
            .ok_or_else(|| crate::NetworkError::Protocol("AabbState missing min".to_string()))?;
        let max = s
            .max
            .ok_or_else(|| crate::NetworkError::Protocol("AabbState missing max".to_string()))?;
        Ok(collision::Aabb::new(
            [min.x, min.y, min.z],
            [max.x, max.y, max.z],
        ))
    }
}
