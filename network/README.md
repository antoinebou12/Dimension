# network

Network crate for Dimension: gRPC, HTTP/REST, and WebSocket transports with Protocol Buffers serialization. Integrates with mathlib, render, collision, and kinematics for multiplayer game networking.

## Features

- `grpc` — gRPC transport (tonic)
- `http` — HTTP/REST transport (axum)
- `websocket` — WebSocket transport for real-time state push
- `full` — grpc + http + websocket
- `simd` — SIMD-accelerated operations
- `parallel` — parallel broadcast and batch serialization
- `server` — server-side components (lobby, state, broadcast)
- `client` — client-side connection and sync

## Usage

```rust
use network::protocol::Vec3;
use network::serialize::binary;

let v = Vec3 { x: 1.0, y: 0.0, z: 0.0 };
let bytes = binary::encode_vec3(&v).unwrap();
```

## Examples

- `just run-network-server` — run the full server (gRPC + HTTP)
- `just run-network-client` — run the client
- `cargo run -p network --example echo` — minimal protobuf encode/decode
