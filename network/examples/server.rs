//! Full server example: gRPC and HTTP/REST.

#![cfg(feature = "server")]

use network::server::ServerConfig;
use network::transport::grpc::{LobbyServerService, LobbyServiceImpl};
use network::transport::http::rest_router;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _ = tracing_subscriber::fmt::try_init();
    let config = ServerConfig::default();

    let http_app = rest_router();
    let http_addr = config.http_addr;
    let grpc_addr = config.grpc_addr;

    tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind(http_addr).await.unwrap();
        axum::serve(listener, http_app).await.unwrap();
    });

    let lobby = LobbyServiceImpl::default();
    Server::builder()
        .add_service(LobbyServerService::new(lobby))
        .serve(grpc_addr)
        .await?;

    Ok(())
}
