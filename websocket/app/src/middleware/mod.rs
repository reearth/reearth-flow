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

pub fn add_middleware<S>(router: Router<S>) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    router.layer(
        ServiceBuilder::new()
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
            .layer(TimeoutLayer::new(Duration::from_secs(10)))
            .layer(add_trace_middleware())
            .layer(middleware::from_fn(auth_middleware))
            .into_inner(),
    )
}
