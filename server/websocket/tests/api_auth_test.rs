/// Integration tests for the HTTP Document API authentication middleware.
///
/// These tests exercise the *actual* `api_auth_layer` middleware through a real
/// Axum router using `tower::ServiceExt::oneshot`, sending real HTTP requests
/// and asserting on real HTTP responses. No mocks, no reimplemented logic.
use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::from_fn_with_state,
    routing::get,
    Router,
};
use tower::ServiceExt;

use websocket::presentation::http::middleware::api_auth_layer;

/// Build a minimal Axum router with the auth middleware wired in.
/// The only route is `GET /test` which returns 200 "ok".
fn test_app(api_secret: Option<String>) -> Router {
    Router::new()
        .route("/test", get(|| async { "ok" }))
        .layer(from_fn_with_state(api_secret, api_auth_layer))
}

// ---------- No secret configured (dev mode) ----------

#[tokio::test]
async fn no_secret_configured_allows_request_without_header() {
    let app = test_app(None);
    let req = Request::builder()
        .uri("/test")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn no_secret_configured_allows_request_with_any_header() {
    let app = test_app(None);
    let req = Request::builder()
        .uri("/test")
        .header("X-API-Secret", "anything")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

// ---------- Secret configured ----------

#[tokio::test]
async fn secret_configured_rejects_missing_header() {
    let app = test_app(Some("test-secret-123".into()));
    let req = Request::builder()
        .uri("/test")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn secret_configured_rejects_wrong_header() {
    let app = test_app(Some("test-secret-123".into()));
    let req = Request::builder()
        .uri("/test")
        .header("X-API-Secret", "wrong-secret")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn secret_configured_rejects_empty_header() {
    let app = test_app(Some("test-secret-123".into()));
    let req = Request::builder()
        .uri("/test")
        .header("X-API-Secret", "")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn secret_configured_allows_correct_header() {
    let app = test_app(Some("test-secret-123".into()));
    let req = Request::builder()
        .uri("/test")
        .header("X-API-Secret", "test-secret-123")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn secret_comparison_is_case_sensitive() {
    let app = test_app(Some("MySecret".into()));
    let req = Request::builder()
        .uri("/test")
        .header("X-API-Secret", "mysecret")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn secret_comparison_rejects_prefix_match() {
    let app = test_app(Some("secret".into()));
    let req = Request::builder()
        .uri("/test")
        .header("X-API-Secret", "secret-plus-extra")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}
