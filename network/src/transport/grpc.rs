//! gRPC transport: Lobby service for room creation, join, leave, list.

use std::net::SocketAddr;
use tonic::transport::Server;

use crate::protocol::proto::lobby_server::{Lobby as LobbyTrait, LobbyServer};

pub use crate::protocol::proto::lobby_client::LobbyClient;
pub use crate::protocol::proto::lobby_server::LobbyServer as LobbyServerService;

/// Lobby service implementation for gRPC.
#[derive(Debug, Default)]
pub struct LobbyServiceImpl {
    /// Placeholder for shared lobby state (rooms, players).
    _state: (),
}

#[tonic::async_trait]
impl LobbyTrait for LobbyServiceImpl {
    async fn create_room(
        &self,
        request: tonic::Request<crate::protocol::proto::CreateRoomRequest>,
    ) -> Result<tonic::Response<crate::protocol::proto::CreateRoomResponse>, tonic::Status> {
        let req = request.into_inner();
        let room_id = format!("room-{}", req.room_name.replace(' ', "-"));
        Ok(tonic::Response::new(
            crate::protocol::proto::CreateRoomResponse {
                room_id: room_id.clone(),
            },
        ))
    }

    async fn join_room(
        &self,
        request: tonic::Request<crate::protocol::proto::JoinRoomRequest>,
    ) -> Result<tonic::Response<crate::protocol::proto::JoinRoomResponse>, tonic::Status> {
        let req = request.into_inner();
        Ok(tonic::Response::new(
            crate::protocol::proto::JoinRoomResponse {
                success: true,
                message: format!("{} joined {}", req.player_name, req.room_id),
            },
        ))
    }

    async fn leave_room(
        &self,
        request: tonic::Request<crate::protocol::proto::LeaveRoomRequest>,
    ) -> Result<tonic::Response<crate::protocol::proto::LeaveRoomResponse>, tonic::Status> {
        let _req = request.into_inner();
        Ok(tonic::Response::new(
            crate::protocol::proto::LeaveRoomResponse { success: true },
        ))
    }

    async fn list_rooms(
        &self,
        _request: tonic::Request<crate::protocol::proto::ListRoomsRequest>,
    ) -> Result<tonic::Response<crate::protocol::proto::ListRoomsResponse>, tonic::Status> {
        Ok(tonic::Response::new(
            crate::protocol::proto::ListRoomsResponse { rooms: vec![] },
        ))
    }
}

/// Spawn a gRPC server listening on `addr` with the Lobby service.
///
/// # Errors
/// Returns [`tonic::transport::Error`] if the server fails to start.
pub async fn run_grpc_server(addr: SocketAddr) -> Result<(), tonic::transport::Error> {
    let lobby = LobbyServiceImpl::default();
    Server::builder()
        .add_service(LobbyServer::new(lobby))
        .serve(addr)
        .await
}
