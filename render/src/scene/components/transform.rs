//! Transform component using mathlib types.

use mathlib::cg::vector3;
use mathlib::cg::{from_euler_angles, new_nonuniform_scaling, new_translation};
use mathlib::math3d::Matrix4f;
use mathlib::Quat4f;

/// Transform: position, rotation (Euler radians or quaternion), scale.
///
/// When [`rotation_quat`](Self::rotation_quat) is `Some`, it is used for the model matrix
/// (avoids gimbal lock); otherwise [`rotation`](Self::rotation) (Euler) is used.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Transform {
    /// Position (x, y, z).
    pub position: [f32; 3],
    /// Rotation (roll, pitch, yaw) in radians. Ignored when [`rotation_quat`](Self::rotation_quat) is `Some`.
    pub rotation: [f32; 3],
    /// Optional rotation as unit quaternion. When set, used by [`to_model_matrix`](Self::to_model_matrix) instead of [`rotation`](Self::rotation).
    #[cfg_attr(feature = "serde", serde(default))]
    pub rotation_quat: Option<Quat4f>,
    /// Scale (x, y, z).
    pub scale: [f32; 3],
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            rotation_quat: None,
            scale: [1.0, 1.0, 1.0],
        }
    }
}

impl Transform {
    /// Build model matrix T * R * S using mathlib CG.
    ///
    /// Uses [`rotation_quat`](Self::rotation_quat) when `Some`, otherwise [`rotation`](Self::rotation) (Euler).
    #[must_use]
    pub fn to_model_matrix(&self) -> Matrix4f {
        let pos = vector3(self.position[0], self.position[1], self.position[2]);
        let scale_vec = vector3(self.scale[0], self.scale[1], self.scale[2]);
        let t = new_translation(&pos);
        let r = self
            .rotation_quat
            .map(|q| q.to_rotation_matrix4())
            .unwrap_or_else(|| {
                from_euler_angles(self.rotation[0], self.rotation[1], self.rotation[2])
            });
        let s = new_nonuniform_scaling(&scale_vec);
        &(&t * &r) * &s
    }

    /// Creates a transform from quaternion rotation and position (unit scale).
    #[must_use]
    pub fn from_position_quat(position: [f32; 3], q: Quat4f) -> Self {
        let (r, p, y) = q.to_euler_angles();
        Self {
            position,
            rotation: [r, p, y],
            rotation_quat: Some(q),
            scale: [1.0, 1.0, 1.0],
        }
    }

    /// Creates a transform from quaternion rotation and position (unit scale).
    #[must_use]
    pub fn from_quaternion(q: &Quat4f, position: [f32; 3]) -> Self {
        Self::from_position_quat(position, *q)
    }

    /// Returns the rotation as a unit quaternion.
    #[must_use]
    pub fn to_quaternion(&self) -> Quat4f {
        Quat4f::from_euler_angles(self.rotation[0], self.rotation[1], self.rotation[2])
    }

    /// Sets rotation from a unit quaternion.
    pub fn set_rotation_quat(&mut self, q: &Quat4f) {
        let (r, p, y) = q.to_euler_angles();
        self.rotation = [r, p, y];
        self.rotation_quat = Some(*q);
    }

    /// Shorthand: transform with the given position (identity rotation, unit scale).
    #[must_use]
    pub fn with_position(x: f32, y: f32, z: f32) -> Self {
        Self {
            position: [x, y, z],
            rotation: [0.0, 0.0, 0.0],
            rotation_quat: None,
            scale: [1.0, 1.0, 1.0],
        }
    }

    /// Shorthand: transform with the given position and Euler rotation (unit scale).
    #[must_use]
    pub fn with_position_rotation(pos: [f32; 3], rot: [f32; 3]) -> Self {
        Self {
            position: pos,
            rotation: rot,
            rotation_quat: None,
            scale: [1.0, 1.0, 1.0],
        }
    }
}
