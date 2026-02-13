//! World: hierarchical scene graph using mathlib Tree.

use mathlib::cg::{matrix4f_identity, transform_point, vector3};
use mathlib::graph::Tree;
use mathlib::math3d::{matrix4f_inverse, Matrix4f};

use super::components::{Primitive, Transform};
use super::entity::EntityId;
use super::node_data::NodeData;

/// World: tree of entities with NodeData.
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct World {
    tree: Tree<NodeData>,
    next_id: usize,
}

impl World {
    /// Create a new world with root at index 0.
    #[must_use]
    pub fn new() -> Self {
        let mut tree = Tree::new(0);
        tree.nodes[0].data = NodeData::default();
        Self { tree, next_id: 1 }
    }

    /// Spawn a new entity as child of `parent`. Returns the new entity id.
    #[must_use]
    pub fn spawn(&mut self, parent: EntityId) -> EntityId {
        let id = self.next_id;
        self.next_id += 1;
        self.tree.add_child(parent.0, id);
        self.tree.nodes[id].data = NodeData::default();
        EntityId(id)
    }

    /// Get node data for an entity.
    #[must_use]
    pub fn get(&self, id: EntityId) -> Option<&NodeData> {
        self.tree.nodes.get(id.0).map(|n| &n.data)
    }

    /// Get mutable node data for an entity.
    pub fn get_mut(&mut self, id: EntityId) -> Option<&mut NodeData> {
        self.tree.nodes.get_mut(id.0).map(|n| &mut n.data)
    }

    /// Set transform for an entity.
    pub fn set_transform(&mut self, id: EntityId, transform: Transform) {
        if let Some(data) = self.get_mut(id) {
            data.transform = transform;
        }
    }

    /// Set primitive for an entity.
    pub fn set_primitive(&mut self, id: EntityId, primitive: Primitive) {
        if let Some(data) = self.get_mut(id) {
            data.primitive = Some(primitive);
        }
    }

    /// Set color for an entity.
    pub fn set_color(&mut self, id: EntityId, color: [f32; 4]) {
        if let Some(data) = self.get_mut(id) {
            data.color = color;
        }
    }

    /// Set material for an entity. Pass `None` for vertex color mode.
    pub fn set_material(&mut self, id: EntityId, material: Option<impl Into<String>>) {
        if let Some(data) = self.get_mut(id) {
            data.material = material.map(|s| s.into());
        }
    }

    /// Reference to the tree for iteration.
    #[must_use]
    pub fn tree(&self) -> &Tree<NodeData> {
        &self.tree
    }

    /// Root entity (tree root index). Use with [`Self::children`] to traverse the tree.
    ///
    /// # Examples
    /// ```
    /// # use render::{EntityId, World};
    /// let world = World::new();
    /// let root = world.root_entity();
    /// let children = world.children(root);
    /// ```
    #[must_use]
    pub fn root_entity(&self) -> EntityId {
        EntityId(self.tree.root)
    }

    /// Children of an entity. Returns an empty vec if the entity does not exist.
    #[must_use]
    pub fn children(&self, id: EntityId) -> Vec<EntityId> {
        self.tree
            .nodes
            .get(id.0)
            .map(|n| n.children.iter().copied().map(EntityId).collect())
            .unwrap_or_default()
    }

    /// Parent of an entity, if any. Root has no parent. Use with [`Self::root_entity`] and [`Self::children`] to walk the hierarchy.
    #[must_use]
    pub fn parent(&self, id: EntityId) -> Option<EntityId> {
        self.tree
            .nodes
            .get(id.0)
            .and_then(|n| n.parent.map(EntityId))
    }

    /// Entities in DFS preorder (root, then descendants). Excludes despawned entities (active = false).
    /// Stable order for UI and iteration. When the `serde` feature is enabled, the tree can be serialized and this structure is preserved.
    #[must_use]
    pub fn entities_dfs(&self) -> Vec<EntityId> {
        self.tree
            .dfs_preorder()
            .into_iter()
            .filter(|&i| self.tree.nodes.get(i).map_or(false, |n| n.data.active))
            .map(EntityId)
            .collect()
    }

    /// Remove one entity from the scene (one at a time). Uses the mathlib tree; marks the node inactive so it is excluded from [`Self::entities_dfs`] and rendering. Root cannot be despawned.
    pub fn despawn(&mut self, id: EntityId) {
        if id.0 == self.tree.root {
            return;
        }
        if let Some(node) = self.tree.nodes.get_mut(id.0) {
            node.data.active = false;
        }
    }

    /// Hard-remove an entity from the tree. Recursively removes descendants, then detaches the entity from its parent. The entity id is invalidated (no longer in the tree). Root cannot be removed.
    ///
    /// # Panics
    /// Panics if `id` is the root entity.
    pub fn remove(&mut self, id: EntityId) {
        if id.0 == self.tree.root {
            panic!("cannot remove root entity");
        }
        let Some(parent) = self.tree.nodes.get(id.0).and_then(|n| n.parent) else {
            return;
        };
        let children = self.children(id);
        for c in children {
            self.remove(c);
        }
        self.tree.remove_child(parent, id.0);
    }

    /// Reparent an entity to a new parent. The entity and its subtree move to the new parent.
    ///
    /// # Panics
    /// Panics if the new parent is a descendant of the entity (would create a cycle).
    pub fn reparent(&mut self, id: EntityId, new_parent: EntityId) {
        self.tree.reparent(id.0, new_parent.0);
    }

    /// World matrix for an entity (product of all local transforms from root to this entity).
    /// Returns `None` if the entity does not exist or is inactive.
    #[must_use]
    pub fn entity_world_matrix(&self, id: EntityId) -> Option<Matrix4f> {
        let node = self.tree.nodes.get(id.0)?;
        if !node.data.active {
            return None;
        }
        let mut path = Vec::new();
        let mut cur = id;
        loop {
            path.push(cur);
            match self.parent(cur) {
                Some(p) => cur = p,
                None => break,
            }
        }
        path.reverse();
        let mut parent = matrix4f_identity();
        for &node_id in &path {
            let node = &self.tree.nodes[node_id.0];
            let local = node.data.transform.to_model_matrix();
            parent = &parent * &local;
        }
        Some(parent)
    }

    /// World-space position of an entity (origin transformed by world matrix).
    /// Returns `None` if the entity does not exist or is inactive.
    #[must_use]
    pub fn entity_world_position(&self, id: EntityId) -> Option<[f32; 3]> {
        let m = self.entity_world_matrix(id)?;
        let origin = vector3(0.0, 0.0, 0.0);
        let p = transform_point(&m, &origin);
        Some([p.get(0), p.get(1), p.get(2)])
    }

    /// Set an entity's local position so that its world position becomes `(x, y, z)`.
    /// No-op if the entity does not exist or is the root (root has no parent).
    pub fn set_entity_world_position(&mut self, id: EntityId, x: f32, y: f32, z: f32) {
        if id.0 == self.tree.root {
            return;
        }
        let Some(parent_id) = self.parent(id) else {
            return;
        };
        let Some(parent_world) = self.entity_world_matrix(parent_id) else {
            return;
        };
        let parent_inv = matrix4f_inverse(&parent_world);
        let world_pt = vector3(x, y, z);
        let local_pt = transform_point(&parent_inv, &world_pt);
        if let Some(data) = self.get_mut(id) {
            data.transform.position = [local_pt.get(0), local_pt.get(1), local_pt.get(2)];
        }
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}
