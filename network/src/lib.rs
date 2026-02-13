//! Network crate for Dimension: gRPC, HTTP/REST, and WebSocket transports.
//!
//! Provides Protocol Buffers serialization and conversion between network
//! messages and mathlib, render, collision, and kinematics types for
//! multiplayer game networking.
//!
//! # Features
//!
//! - `grpc` — gRPC transport (tonic)
//! - `http` — HTTP/REST transport (axum)
//! - `websocket` — WebSocket transport for real-time state push
//! - `full` — grpc + http + websocket
//! - `simd` — SIMD-accelerated operations
//! - `parallel` — parallel broadcast and batch serialization
//! - `server` — server-side components (lobby, state, broadcast)
//! - `client` — client-side connection and sync
//!
//! # Examples
//!
//! ```
//! use network::protocol::Vec3;
//! use network::serialize::binary;
//! let v = Vec3 { x: 1.0, y: 0.0, z: 0.0 };
//! let bytes = binary::encode_vec3(&v).unwrap();
//! ```

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

pub mod error;
pub mod protocol;
pub mod serialize;
pub mod utils;

#[cfg(any(feature = "grpc", feature = "http", feature = "websocket"))]
pub mod transport;

#[cfg(feature = "server")]
pub mod server;

#[cfg(feature = "client")]
pub mod client;

pub use error::NetworkError;
pub use protocol::{convert, messages};
pub use serialize::{binary, json};
pub use utils::{
    batch, clock, compression, conflict, delta, event, order, pool, reconnect, throttle,
};
