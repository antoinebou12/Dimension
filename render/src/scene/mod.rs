//! Scene layer: World, entities, components.
//!
//! The world is a hierarchical scene graph (mathlib [`Tree`](mathlib::graph::Tree)): each entity has a parent and zero or more children. Use [`World::root_entity`], [`World::children`], [`World::parent`], and [`World::entities_dfs`] to traverse. With the `serde` feature, world and components can be serialized to a compact binary format.

mod entity;
mod node_data;
mod world;

pub mod components;

pub use components::{CurvePoint, Primitive, Primitive2D, Primitive3D, Transform};
pub use entity::EntityId;
pub use node_data::NodeData;
pub use world::World;
