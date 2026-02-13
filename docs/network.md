# Network Crate

The `network` crate provides gRPC, HTTP/REST, and WebSocket transports with Protocol Buffers serialization for multiplayer game networking in the Dimension workspace.

## Architecture

- **Protocol** — Protobuf messages (Vec3, Transform, EntityState, WorldSnapshot, WorldDelta, GameCommand, Lobby)
- **Transports** — gRPC (tonic), HTTP/REST (axum), WebSocket (axum ws)
- **Serialization** — Prost for binary, serde_json for JSON
- **Server** — Lobby service, shared game state, broadcast channel

## Build and Run

```bash
just build-network
just test-network
just bench-network
just run-network-server   # gRPC on :50051, HTTP on :3000
just run-network-client
```

## Features

| Feature     | Purpose                                |
|-------------|----------------------------------------|
| `grpc`      | gRPC transport (tonic)                 |
| `http`      | HTTP/REST (axum)                       |
| `websocket` | WebSocket real-time state push         |
| `server`    | Lobby, state, broadcast                |
| `client`    | Connection manager, state sync         |
| `simd`      | SIMD-accelerated ops                   |
| `parallel`  | Parallel broadcast and serialization   |
