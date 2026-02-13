//! HTTP transport tests.

#![cfg(feature = "http")]

use axum::body::Body;
use axum::http::{Request, StatusCode};
use network::transport::http::rest_router;
use tower::util::ServiceExt;

#[tokio::test]
async fn http_health() {
    let app = rest_router();
    let req = Request::builder()
        .uri("/health")
        .body(Body::empty())
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}
