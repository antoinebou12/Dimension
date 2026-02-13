//! WebSocket transport: real-time state broadcast.

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::broadcast;

/// Shared state for WebSocket handler.
#[derive(Clone)]
pub struct WsState {
    /// Broadcast channel for state updates.
    pub tx: broadcast::Sender<Vec<u8>>,
}

impl WsState {
    /// Create new WebSocket state with given channel capacity.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Self { tx }
    }

    /// Create from existing broadcast sender.
    #[must_use]
    pub fn from_sender(tx: broadcast::Sender<Vec<u8>>) -> Self {
        Self { tx }
    }
}

/// WebSocket upgrade handler. Accepts connections and spawns a task to receive/send.
pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<Arc<WsState>>) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<WsState>) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.tx.subscribe();

    let send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Binary(msg.into())).await.is_err() {
                break;
            }
        }
    });

    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Binary(data) = msg {
                let _ = state.tx.send(data.to_vec());
            }
        }
    });

    let _ = tokio::join!(send_task, recv_task);
}
