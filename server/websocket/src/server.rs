use axum::{
    body::Body,
    extract::{Path, State, WebSocketUpgrade},
    http::Response,
    routing::get,
    Router,
};
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::cors::{Any, CorsLayer};

use google_cloud_storage::{
    client::Client,
    http::buckets::insert::{BucketCreationConfig, InsertBucketRequest},
};
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;

#[cfg(feature = "auth")]
use crate::AuthQuery;
use crate::{doc::document_routes, AppState};
use anyhow::Result;
#[cfg(feature = "auth")]
use axum::extract::Query;

#[derive(Clone)]
struct ServerState {
    app_state: Arc<AppState>,
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("signal received, starting graceful shutdown");
}

pub async fn ensure_bucket(client: &Client, bucket_name: &str) -> Result<()> {
    let bucket = BucketCreationConfig {
        location: "US".to_string(),
        ..Default::default()
    };
    let request = InsertBucketRequest {
        name: bucket_name.to_string(),
        bucket,
        ..Default::default()
    };

    match client.insert_bucket(&request).await {
        Ok(_) => Ok(()),
        Err(e) if e.to_string().contains("already exists") => Ok(()),
        Err(e) => Err(e.into()),
    }
}

pub async fn start_server(state: Arc<AppState>, port: &str, config: &crate::Config) -> Result<()> {
    let addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(&addr).await?;

    info!("Starting server on {}", addr);

    let server_state = ServerState {
        app_state: state.clone(),
    };

    let ws_router = Router::new()
        .route("/{doc_id}", get(ws_handler))
        .with_state(server_state);

    let app = Router::new()
        .merge(ws_router)
        .nest("/api", document_routes())
        .with_state(state)
        .layer(
            ServiceBuilder::new()
                .layer({
                    let origins: Vec<_> = config
                        .app
                        .origins
                        .iter()
                        .map(|s| s.parse().unwrap())
                        .collect();

                    CorsLayer::new()
                        .allow_origin(origins)
                        .allow_methods(Any)
                        .allow_headers(Any)
                })
                .layer(
                    CompressionLayer::new()
                        .compress_when(tower_http::compression::predicate::SizeAbove::new(1024)),
                ),
        );

    info!("WebSocket endpoint available at ws://{}/[doc_id]", addr);
    info!(
        "HTTP API endpoints available at http://{}/api/document/...",
        addr
    );
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

#[cfg(feature = "auth")]
async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(doc_id): Path<String>,
    Query(query): Query<AuthQuery>,
    State(state): State<ServerState>,
) -> Response<Body> {
    crate::interface::ws::ws_handler(ws, Path(doc_id), Query(query), State(state.app_state)).await
}

#[cfg(not(feature = "auth"))]
async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(doc_id): Path<String>,
    State(state): State<ServerState>,
) -> Response<Body> {
    crate::ws::ws_handler(ws, Path(doc_id), State(state.app_state)).await
}
