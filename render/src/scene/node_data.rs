//! Node data stored in world tree nodes.

use super::components::{Primitive, Transform};

/// Data attached to each entity (tree node).
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NodeData {
    /// Local transform.
    pub transform: Transform,
    /// Shape to render; None = no draw.
    pub primitive: Option<Primitive>,
    /// RGBA color.
    pub color: [f32; 4],
    /// Material name; None = vertex color mode.
    #[cfg_attr(feature = "serde", serde(default))]
    pub material: Option<String>,
    /// If false, entity is despawned (excluded from iteration and rendering). Root cannot be despawned.
    pub active: bool,
}

impl Default for NodeData {
    fn default() -> Self {
        Self {
            transform: Transform::default(),
            primitive: None,
            color: [1.0, 1.0, 1.0, 1.0],
            material: None,
            active: true,
        }
    }
}
