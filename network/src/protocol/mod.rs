//! Protocol messages and conversion between network types and Dimension crates.
//!
//! Generated Protocol Buffers types and conversion traits for mathlib, render,
//! collision, and kinematics types.

/// Generated protobuf types (from dimension.proto).
pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/network.rs"));
}

pub mod convert;
pub mod messages;

pub use convert::*;
pub use messages::*;
pub use proto::*;
