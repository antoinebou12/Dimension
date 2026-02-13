//! Client example: connect to server via gRPC and HTTP.

#![cfg(feature = "client")]

use network::client::ConnectionConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = ConnectionConfig::default();
    println!("Client configured for {}", config.base_url);
    Ok(())
}
