mod auth;
mod trace;
use auth::auth_middleware;
use axum::{error_handling::HandleErrorLayer, http::StatusCode, middleware, BoxError, Router};

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
                if err.is::<tower::timeout::error::Elapsed>() {
                    StatusCode::REQUEST_TIMEOUT
                } else {
                    StatusCode::INTERNAL_SERVER_ERROR
                }
            }))
            .layer(TimeoutLayer::new(Duration::from_secs(10)))
            .layer(add_trace_middleware())
            .layer(middleware::from_fn(auth_middleware))
            .into_inner(),
    )
}
