//! Tree structure with explicit parent/children and BFS spanning tree construction.
//!
//! Provides [`Tree`] and [`Node`] for representing rooted trees. Trees can be built
//! manually with [`Tree::new`] and [`Tree::add_child`], or from a graph via
//! [`Tree::from_bfs_spanning_tree`].

use std::collections::VecDeque;

use crate::graph::types::Graph;

/// A node in a tree: optional parent, list of child indices, and optional payload.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Node<T = ()> {
    /// Parent node index, if any. Root has `None`.
    pub parent: Option<usize>,
    /// Child node indices.
    pub children: Vec<usize>,
    /// Optional payload.
    pub data: T,
}

impl<T> Node<T> {
    /// Creates a node with no parent (root), no children, and the given data.
    #[inline]
    #[must_use]
    pub fn new(data: T) -> Self {
        Self {
            parent: None,
            children: Vec::new(),
            data,
        }
    }

    /// Returns whether this node is a leaf (no children).
    #[inline]
    #[must_use]
    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }
}

impl<T: Default> Default for Node<T> {
    fn default() -> Self {
        Self {
            parent: None,
            children: Vec::new(),
            data: T::default(),
        }
    }
}

impl Node<()> {
    /// Creates a root node with unit data.
    #[inline]
    #[must_use]
    pub fn root() -> Self {
        Self::new(())
    }
}

/// A rooted tree: nodes indexed by position, with explicit parent/child links.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Tree<T = ()> {
    /// Nodes in the tree. Index corresponds to graph node id when built from a graph.
    pub nodes: Vec<Node<T>>,
    /// Root node index.
    pub root: usize,
}

impl<T: Default> Tree<T> {
    /// Creates a tree with a single root node at index `root`.
    ///
    /// # Panics
    /// Panics if the tree is later used with indices that require more capacity.
    #[must_use]
    pub fn new(root: usize) -> Self {
        let mut nodes = Vec::new();
        nodes.resize_with(root + 1, Node::default);
        nodes[root] = Node {
            parent: None,
            children: Vec::new(),
            data: T::default(),
        };
        Self { nodes, root }
    }

    /// Ensures capacity for node index `i` and returns the node at that index.
    fn ensure_node(&mut self, i: usize) -> &mut Node<T> {
        if i >= self.nodes.len() {
            self.nodes.resize_with(i + 1, Node::default);
        }
        &mut self.nodes[i]
    }

    /// Adds `child` as a child of `parent`, creating nodes as needed.
    ///
    /// # Panics
    /// Panics if `parent == child` (would create a self-loop).
    pub fn add_child(&mut self, parent: usize, child: usize) {
        assert!(parent != child, "cannot add self as child");
        self.ensure_node(parent).children.push(child);
        let child_node = self.ensure_node(child);
        child_node.parent = Some(parent);
    }

    /// Removes `child` from `parent`'s children. The child's parent is set to `None`;
    /// descendants of `child` remain attached (subtree is detached, not deleted).
    ///
    /// # Panics
    /// Panics if `child == self.root` (root cannot be removed).
    pub fn remove_child(&mut self, parent: usize, child: usize) {
        assert!(child != self.root, "cannot remove root");
        if let Some(p) = self.nodes.get_mut(parent) {
            p.children.retain(|&c| c != child);
        }
        if let Some(c) = self.nodes.get_mut(child)
            && c.parent == Some(parent)
        {
            c.parent = None;
        }
    }

    /// Moves `node` to be a child of `new_parent`. Removes `node` from its current
    /// parent's children, then adds it to `new_parent`'s children and updates
    /// `node.parent`.
    ///
    /// # Panics
    /// Panics if `new_parent` is a descendant of `node` (would create a cycle).
    pub fn reparent(&mut self, node: usize, new_parent: usize) {
        assert!(
            !self.is_descendant_of(new_parent, node),
            "reparent would create cycle: new_parent is descendant of node"
        );
        if let Some(old_parent) = self.nodes.get(node).and_then(|n| n.parent) {
            if old_parent == new_parent {
                return;
            }
            if let Some(p) = self.nodes.get_mut(old_parent) {
                p.children.retain(|&c| c != node);
            }
        }
        self.ensure_node(new_parent).children.push(node);
        self.ensure_node(node).parent = Some(new_parent);
    }

    /// Returns whether `descendant` is in the subtree rooted at `ancestor` (or is `ancestor`).
    fn is_descendant_of(&self, descendant: usize, ancestor: usize) -> bool {
        if descendant == ancestor {
            return true;
        }
        let mut cur = descendant;
        while let Some(node) = self.nodes.get(cur) {
            let Some(p) = node.parent else {
                return false;
            };
            if p == cur {
                return false;
            }
            cur = p;
            if cur == ancestor {
                return true;
            }
        }
        false
    }

    /// Builds a BFS spanning tree from `graph` starting at `source`.
    ///
    /// Treats the graph as undirected. Includes all nodes reachable from `source`.
    /// Node indices in the tree correspond to graph node indices.
    #[must_use]
    pub fn from_bfs_spanning_tree(graph: &Graph, source: usize) -> Self {
        let n = graph.num_nodes();
        if source >= n {
            return Self {
                nodes: Vec::new(),
                root: source,
            };
        }
        let mut parent: Vec<Option<usize>> = vec![None; n];
        let mut queue = VecDeque::new();
        queue.push_back(source);
        parent[source] = Some(source); // sentinel to mark visited; root's real parent is None
        while let Some(u) = queue.pop_front() {
            for &(v, _) in graph.neighbors(u) {
                if parent[v].is_none() {
                    parent[v] = Some(u);
                    queue.push_back(v);
                }
            }
            for &(v, _) in graph.in_neighbors(u) {
                if parent[v].is_none() {
                    parent[v] = Some(u);
                    queue.push_back(v);
                }
            }
        }
        // Build nodes
        let mut nodes: Vec<Node<T>> = (0..n)
            .map(|i| {
                let p = parent[i];
                let parent_idx = if i == source { None } else { p };
                Node {
                    parent: parent_idx,
                    children: Vec::new(),
                    data: T::default(),
                }
            })
            .collect();
        for (v, p_opt) in parent.iter().enumerate() {
            if let Some(par) = p_opt
                && *par != v
            {
                nodes[*par].children.push(v);
            }
        }
        Self {
            nodes,
            root: source,
        }
    }

    /// Returns BFS order (nodes level by level).
    #[must_use]
    pub fn bfs_order(&self) -> Vec<usize> {
        if self.root >= self.nodes.len() {
            return Vec::new();
        }
        let mut order = Vec::with_capacity(self.nodes.len());
        let mut queue = VecDeque::new();
        queue.push_back(self.root);
        while let Some(u) = queue.pop_front() {
            order.push(u);
            for &c in &self.nodes[u].children {
                queue.push_back(c);
            }
        }
        order
    }

    /// Returns DFS preorder (visit node before its descendants).
    #[must_use]
    pub fn dfs_preorder(&self) -> Vec<usize> {
        if self.root >= self.nodes.len() {
            return Vec::new();
        }
        let mut order = Vec::with_capacity(self.nodes.len());
        let mut stack = vec![self.root];
        while let Some(u) = stack.pop() {
            order.push(u);
            for &c in self.nodes[u].children.iter().rev() {
                stack.push(c);
            }
        }
        order
    }

    /// Returns DFS postorder (visit node after its descendants).
    #[must_use]
    pub fn dfs_postorder(&self) -> Vec<usize> {
        if self.root >= self.nodes.len() {
            return Vec::new();
        }
        let mut order = Vec::with_capacity(self.nodes.len());
        let mut stack = vec![(self.root, false)];
        while let Some((u, children_done)) = stack.pop() {
            if children_done {
                order.push(u);
                continue;
            }
            stack.push((u, true));
            for &c in self.nodes[u].children.iter().rev() {
                stack.push((c, false));
            }
        }
        order
    }

    /// Returns the number of nodes in the tree.
    #[inline]
    #[must_use]
    pub fn num_nodes(&self) -> usize {
        self.nodes.len()
    }
}

impl<T> Tree<T> {
    /// Returns the path from root to `node` (root first). Empty if `node` is out of range.
    #[must_use]
    pub fn path_from_root(&self, node: usize) -> Vec<usize> {
        let mut path = Vec::new();
        let mut cur = node;
        while let Some(n) = self.nodes.get(cur) {
            path.push(cur);
            match n.parent {
                Some(p) => cur = p,
                None => break,
            }
        }
        path.reverse();
        path
    }
}

/// Maps path indices to flat offsets in a traversal order.
///
/// Given a `path` (e.g. root-to-end-effector), a `traversal_order` (e.g. DFS preorder),
/// and a `node_size` function (e.g. DOF count), returns a vec of flat indices such that
/// for each path node, the mapping contains `node_size(node)` consecutive offsets.
#[must_use]
pub fn path_to_traversal_mapping(
    path: &[usize],
    traversal_order: &[usize],
    node_size: impl Fn(usize) -> usize,
) -> Vec<usize> {
    let mut offsets = std::collections::HashMap::with_capacity(traversal_order.len());
    let mut offset = 0;
    for &idx in traversal_order {
        offsets.insert(idx, offset);
        offset += node_size(idx);
    }

    let mut mapping = Vec::new();
    for &idx in path {
        let size = node_size(idx);
        if let Some(&base) = offsets.get(&idx) {
            for d in 0..size {
                mapping.push(base + d);
            }
        }
    }
    mapping
}
