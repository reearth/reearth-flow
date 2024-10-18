use axum::{error_handling::HandleErrorLayer, Router};
use std::time::Duration;
use tower::{timeout::TimeoutLayer, ServiceBuilder};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};

use crate::handler::handle_error;

pub fn add_middleware<S>(router: Router<S>) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    router.layer(
        ServiceBuilder::new()
            .layer(HandleErrorLayer::new(handle_error))
            .layer(TimeoutLayer::new(Duration::from_secs(10)))
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(DefaultMakeSpan::default().include_headers(true)),
            )
            .into_inner(),
    )
}
