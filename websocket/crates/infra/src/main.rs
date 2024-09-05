use std::{net::SocketAddr, sync::Arc, time::Duration};

use axum::{error_handling::HandleErrorLayer, routing::get, Router};
use socket::{
    handler::{handle_error, handle_upgrade},
    state::AppState,
};
use tower::timeout::TimeoutLayer;
use tower::ServiceBuilder;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
mod socket;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "trace,tower_http=debug,".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let state = Arc::new(AppState::default());
    let state_err = state.clone();
    let app = Router::new()
        .route("/:room", get(handle_upgrade))
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(move |method, uri, err| {
                    handle_error(method, uri, err, state_err)
                }))
                .layer(TimeoutLayer::new(Duration::from_secs(10)))
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(DefaultMakeSpan::default().include_headers(true)),
                )
                .into_inner(),
        )
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8000")
        .await
        .unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
}
