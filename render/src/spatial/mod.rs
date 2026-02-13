//! Spatial data structures for visibility and picking.
//!
//! Re-exports BSP tree from the **collision** crate. Used for frustum visibility
//! and ray traversal in picking.

pub use collision::{BspNode, BspTree};
