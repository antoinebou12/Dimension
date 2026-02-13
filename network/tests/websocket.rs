//! WebSocket transport tests.

#![cfg(feature = "websocket")]

use axum::body::Body;
use axum::http::Request;
use axum::routing::get;
use axum::Router;
use network::transport::websocket::{ws_handler, WsState};
use std::sync::Arc;

#[tokio::test]
async fn websocket_state_creation() {
    let state = WsState::new(16);
    let _ = state.tx.send(vec![1, 2, 3]);
}
