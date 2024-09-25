use std::{net::SocketAddr, sync::Arc, time::Duration};

use axum::{error_handling::HandleErrorLayer, middleware::from_fn, routing::get, Router};
use infra::{auth_middleware, JwtValidator};
use socket::{
    handler::{handle_error, handle_upgrade},
    state::AppState,
};

use common::Config;
use services::AuthServiceClient;

use tower::timeout::TimeoutLayer;
use tower::ServiceBuilder;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod socket;

#[tokio::main]
async fn main() {
    init_tracing_subscriber();

    let config = match Config::from_env() {
        Ok(cfg) => cfg,
        Err(e) => {
            tracing::error!("Failed to load configuration: {}", e);
            return;
        }
    };

    let auth_client = match AuthServiceClient::new(&config.auth_service_url) {
        Ok(client) => client,
        Err(e) => {
            tracing::error!("Failed to create auth client: {}", e);
            return;
        }
    };

    let jwt_validator = JwtValidator::new(auth_client, Duration::from_secs(300));

    let state = Arc::new(AppState::default());
    let state_err = state.clone();
    let app = Router::new()
        .route("/:room", get(handle_upgrade))
        .layer(from_fn(move |req, next| {
            let jwt_validator = jwt_validator.clone();
            auth_middleware::<axum::body::Body>(jwt_validator, req, next)
        }))
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(move |method, uri, err| {
                    handle_error(method, uri, err, state_err.clone())
                }))
                .layer(TimeoutLayer::new(Duration::from_secs(10)))
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(DefaultMakeSpan::default().include_headers(true)),
                )
                .into_inner(),
        )
        .with_state(state);

    let listener = match tokio::net::TcpListener::bind(&config.server_addr).await {
        Ok(listener) => listener,
        Err(e) => {
            tracing::error!("Failed to bind to address {}: {}", &config.server_addr, e);
            return;
        }
    };

    let local_addr = match listener.local_addr() {
        Ok(addr) => addr,
        Err(e) => {
            tracing::error!("Failed to get local address: {}", e);
            return;
        }
    };

    tracing::debug!("listening on {}", local_addr);

    if let Err(e) = axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    {
        tracing::error!("server error: {}", e);
    }
}

#[inline]
fn init_tracing_subscriber() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "trace,tower_http=debug,".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}
