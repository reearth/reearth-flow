mod auth;
mod trace;
use auth::auth_middleware;
use axum::{
    error_handling::HandleErrorLayer, http::StatusCode, middleware, response::IntoResponse,
    BoxError, Json, Router,
};
use serde_json::json;

use std::time::Duration;
use tower::timeout::TimeoutLayer;
use tower::ServiceBuilder;
pub use trace::add_trace_middleware;

pub fn add_middleware<S>(router: Router<S>, include_auth: bool) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    let builder = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|err: BoxError| async move {
            let (status, message) = if err.is::<tower::timeout::error::Elapsed>() {
                (StatusCode::REQUEST_TIMEOUT, "Request timed out")
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            };

            let json_response = json!({
                "error": {
                    "message": message,
                    "code": status.as_u16(),
                    "details": err.to_string()
                }
            });

            (status, Json(json_response)).into_response()
        }))
        .layer(TimeoutLayer::new(Duration::from_secs(3)))
        .layer(add_trace_middleware());

    let router = if include_auth {
        router.layer(middleware::from_fn(auth_middleware))
    } else {
        router
    };

    router.layer(builder.into_inner())
}
#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use serde_json::Value;
    use std::time::Duration;
    use tower::ServiceExt; // for `oneshot`

    #[tokio::test]
    async fn test_timeout_handling() {
        let router = add_middleware(
            Router::new().route(
                "/timeout",
                axum::routing::get(|| async {
                    tokio::time::sleep(Duration::from_secs(20)).await;
                    "This should timeout"
                }),
            ),
            false,
        );

        let response = router
            .oneshot(
                Request::builder()
                    .uri("/timeout")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::REQUEST_TIMEOUT);

        let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["error"]["message"], "Request timed out");
    }

    #[tokio::test]
    async fn test_auth_middleware() {
        let router = add_middleware(
            Router::new().route(
                "/protected",
                axum::routing::get(|| async { "Protected resource" }),
            ),
            true,
        );

        // Simulate unauthorized request
        let response = router
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
