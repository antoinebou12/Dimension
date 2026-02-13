//! Armature: tree of joints with forward kinematics, pack/unpack, reroot.

use mathlib::cg::matrix4f_translation;
use mathlib::graph::{Node, Tree, path_to_traversal_mapping};
use mathlib::{Matrix4f, Vector3f};

use crate::joints::{
    Fixed2dJoint, FixedJoint, Joint, Prismatic2dJoint, PrismaticJoint, Revolute2dJoint,
    RevoluteJoint, SphericalJoint,
};

/// Joint variant for armature nodes.
#[derive(Clone, Debug)]
pub enum JointVariant {
    /// Fixed (0 DOF).
    Fixed(FixedJoint),
    /// Revolute (1 DOF).
    Revolute(RevoluteJoint),
    /// Prismatic (1 DOF).
    Prismatic(PrismaticJoint),
    /// Spherical (3 DOF).
    Spherical(SphericalJoint),
    /// Fixed in 2D (0 DOF, XY plane).
    Fixed2d(Fixed2dJoint),
    /// Revolute in 2D (1 DOF, rotation about Z).
    Revolute2d(Revolute2dJoint),
    /// Prismatic in 2D (1 DOF, slide in XY).
    Prismatic2d(Prismatic2dJoint),
}

impl Default for JointVariant {
    fn default() -> Self {
        Self::Fixed(FixedJoint::default())
    }
}

macro_rules! dispatch_joint {
    ($self:expr, $method:ident $(, $args:expr)*) => {
        match $self {
            JointVariant::Fixed(j) => j.$method($($args),*),
            JointVariant::Revolute(j) => j.$method($($args),*),
            JointVariant::Prismatic(j) => j.$method($($args),*),
            JointVariant::Spherical(j) => j.$method($($args),*),
            JointVariant::Fixed2d(j) => j.$method($($args),*),
            JointVariant::Revolute2d(j) => j.$method($($args),*),
            JointVariant::Prismatic2d(j) => j.$method($($args),*),
        }
    };
}

impl JointVariant {
    /// Local transform from the joint.
    #[must_use]
    pub fn local_transform(&self) -> Matrix4f {
        dispatch_joint!(self, local_transform)
    }

    /// Degrees of freedom.
    #[must_use]
    pub fn dof_count(&self) -> usize {
        dispatch_joint!(self, dof_count)
    }

    /// Pack state into slice.
    pub fn pack(&self, out: &mut [f32]) {
        dispatch_joint!(self, pack, out)
    }

    /// Unpack state from slice.
    pub fn unpack(&mut self, data: &[f32]) {
        dispatch_joint!(self, unpack, data)
    }

    /// Optional angle limits (min, max) in radians for revolute joints. Returns `None` for non-revolute or when unconstrained.
    #[must_use]
    pub fn angle_limits(&self) -> Option<(f32, f32)> {
        match self {
            Self::Revolute(r) => r.angle_min.zip(r.angle_max),
            Self::Revolute2d(r) => r.angle_min.zip(r.angle_max),
            _ => None,
        }
    }
}

/// Data stored per node in the armature.
#[derive(Clone, Debug)]
pub struct JointData {
    /// Joint type and parameters.
    pub joint: JointVariant,
    /// World transform (computed by update_kinematics).
    pub world_transform: Matrix4f,
    /// Optional name.
    pub name: Option<String>,
    /// Whether this is an end-effector for IK.
    pub is_end_effector: bool,
}

impl Default for JointData {
    fn default() -> Self {
        Self {
            joint: JointVariant::default(),
            world_transform: mathlib::cg::matrix4f_identity(),
            name: None,
            is_end_effector: false,
        }
    }
}

impl JointData {
    /// Creates joint data with default world transform.
    #[must_use]
    pub fn new(joint: JointVariant) -> Self {
        Self {
            world_transform: mathlib::cg::matrix4f_identity(),
            joint,
            name: None,
            is_end_effector: false,
        }
    }

    /// Sets the name.
    #[must_use]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Marks as end-effector.
    #[must_use]
    pub fn with_end_effector(mut self, yes: bool) -> Self {
        self.is_end_effector = yes;
        self
    }
}

/// Armature: tree of joints with forward kinematics.
#[derive(Clone, Debug)]
pub struct Armature {
    /// Tree of joint data.
    pub tree: Tree<JointData>,
    /// DFS preorder for consistent traversal.
    dfs_order: Vec<usize>,
}

impl Armature {
    /// Creates an armature with a root node.
    #[must_use]
    pub fn new(root_data: JointData) -> Self {
        let mut tree = Tree::new(0);
        tree.nodes[0] = Node {
            parent: None,
            children: Vec::new(),
            data: root_data,
        };
        let dfs_order = tree.dfs_preorder();
        Self { tree, dfs_order }
    }

    /// Adds a child joint to a parent.
    pub fn add_child(&mut self, parent_idx: usize, child_idx: usize, data: JointData) {
        self.tree.add_child(parent_idx, child_idx);
        let need = child_idx + 1;
        while self.tree.nodes.len() < need {
            self.tree.nodes.push(Node {
                parent: None,
                children: Vec::new(),
                data: JointData::new(JointVariant::Fixed(FixedJoint::default())),
            });
        }
        self.tree.nodes[child_idx].data = data;
        self.dfs_order = self.tree.dfs_preorder();
    }

    /// Updates all world transforms via forward kinematics (parent * local).
    pub fn update_kinematics(&mut self) {
        let order = self.tree.dfs_preorder();
        for &idx in &order {
            let (parent_m, local) = {
                let node = &self.tree.nodes[idx];
                let local = node.data.joint.local_transform();
                let parent_m = node
                    .parent
                    .map(|p| self.tree.nodes[p].data.world_transform.clone());
                (parent_m, local)
            };
            let world = match parent_m {
                Some(p) => &p * &local,
                None => local,
            };
            self.tree.nodes[idx].data.world_transform = world;
        }
    }

    /// Extracts end-effector position (translation part of world transform).
    #[must_use]
    pub fn end_effector_position(&self, node_idx: usize) -> Vector3f {
        matrix4f_translation(&self.tree.nodes[node_idx].data.world_transform)
    }

    /// Packs all joint DOFs into a flat vector (DFS order).
    #[must_use]
    pub fn pack(&self) -> Vec<f32> {
        let mut out = Vec::new();
        for &idx in &self.dfs_order {
            let dof = self.tree.nodes[idx].data.joint.dof_count();
            let start = out.len();
            out.resize(start + dof, 0.0);
            self.tree.nodes[idx].data.joint.pack(&mut out[start..]);
        }
        out
    }

    /// Unpacks joint state from a flat vector (DFS order).
    pub fn unpack(&mut self, data: &[f32]) {
        let mut offset = 0;
        for &idx in &self.dfs_order {
            let dof = self.tree.nodes[idx].data.joint.dof_count();
            if offset + dof <= data.len() {
                self.tree.nodes[idx]
                    .data
                    .joint
                    .unpack(&data[offset..offset + dof]);
            }
            offset += dof;
        }
        self.update_kinematics();
    }

    /// Returns the path from root to the given node (root first, then ancestors to node).
    #[must_use]
    pub fn path_to(&self, end_idx: usize) -> Vec<usize> {
        self.tree.path_from_root(end_idx)
    }

    /// Returns DFS order for IK.
    #[must_use]
    pub fn dfs_order(&self) -> &[usize] {
        &self.dfs_order
    }

    /// Maps path DOF indices to DFS (pack) DOF indices.
    ///
    /// IK computes `d_theta` in path order (root to end-effector), but `pack()`
    /// returns DOFs in DFS preorder. For branched trees these differ, so this
    /// mapping is needed to apply IK updates to the correct joints.
    ///
    /// # Example
    ///
    /// ```text
    /// Tree: root(0) -> child(1), child(2)
    /// DFS order: [0, 1, 2]
    /// path_to(2): [0, 2]  // skips node 1
    ///
    /// If each joint has 1 DOF:
    /// - path DOF 0 (node 0) -> DFS DOF 0
    /// - path DOF 1 (node 2) -> DFS DOF 2
    /// ```
    #[must_use]
    pub fn path_to_dfs_dof_mapping(&self, path: &[usize]) -> Vec<usize> {
        path_to_traversal_mapping(path, &self.dfs_order, |idx| {
            self.tree.nodes[idx].data.joint.dof_count()
        })
    }

    /// Returns the tree.
    #[must_use]
    pub fn tree(&self) -> &Tree<JointData> {
        &self.tree
    }

    /// Returns mutable tree.
    pub fn tree_mut(&mut self) -> &mut Tree<JointData> {
        &mut self.tree
    }
}
