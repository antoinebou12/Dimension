//! gRPC transport tests.

#![cfg(feature = "grpc")]

use network::protocol::proto::lobby_client::LobbyClient;
use network::protocol::proto::lobby_server::LobbyServer;
use network::protocol::proto::CreateRoomRequest;
use network::transport::grpc::LobbyServiceImpl;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tonic::transport::{Endpoint, Server};

#[tokio::test]
async fn grpc_create_room() {
    let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let listener = TcpListener::bind(addr).await.unwrap();
    let addr = listener.local_addr().unwrap();

    let server = Server::builder()
        .add_service(LobbyServer::new(LobbyServiceImpl::default()))
        .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(listener));

    tokio::spawn(server);

    let channel = Endpoint::from_shared(format!("http://{addr}"))
        .unwrap()
        .connect()
        .await
        .unwrap();
    let mut client = LobbyClient::new(channel);

    let req = tonic::Request::new(CreateRoomRequest {
        room_name: "test-room".to_string(),
        max_players: 4,
    });
    let res = client.create_room(req).await.unwrap();
    let res = res.into_inner();
    assert!(res.room_id.starts_with("room-"));
}
