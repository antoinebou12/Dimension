//! HTTP/REST transport: game/room CRUD, health check.

use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};

/// Health check response.
#[derive(serde::Serialize)]
pub struct HealthResponse {
    /// Status string.
    pub status: String,
}

/// Create the REST API router (no shared state).
#[must_use]
pub fn rest_router() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/rooms", get(list_rooms))
        .route("/rooms", post(create_room))
        .route("/rooms/{room_id}/join", post(join_room))
}

async fn health() -> impl IntoResponse {
    Json(HealthResponse {
        status: "ok".to_string(),
    })
}

async fn list_rooms() -> impl IntoResponse {
    Json(serde_json::json!({ "rooms": [] }))
}

async fn create_room(Json(body): Json<serde_json::Value>) -> impl IntoResponse {
    let name = body
        .get("room_name")
        .and_then(|v| v.as_str())
        .unwrap_or("default");
    let room_id = format!("room-{}", name.replace(' ', "-"));
    (
        StatusCode::CREATED,
        Json(serde_json::json!({ "room_id": room_id })),
    )
}

async fn join_room(
    Path(room_id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let player = body
        .get("player_name")
        .and_then(|v| v.as_str())
        .unwrap_or("player");
    Json(serde_json::json!({
        "success": true,
        "message": format!("{} joined {}", player, room_id)
    }))
}
