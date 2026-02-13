//! Server configuration.

use std::net::SocketAddr;

/// Server configuration: ports, features, limits.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// gRPC listen address.
    pub grpc_addr: SocketAddr,
    /// HTTP/REST listen address.
    pub http_addr: SocketAddr,
    /// WebSocket broadcast channel capacity.
    pub broadcast_capacity: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            grpc_addr: ([127, 0, 0, 1], 50051).into(),
            http_addr: ([127, 0, 0, 1], 3000).into(),
            broadcast_capacity: 64,
        }
    }
}
