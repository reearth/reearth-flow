use axum::{
    body::Body,
    extract::{Path, State, WebSocketUpgrade},
    http::Response,
    routing::get,
    Router,
};

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
#[cfg(feature = "auth")]
use axum::extract::Query;

#[derive(Clone)]
struct ServerState {
    app_state: Arc<AppState>,
}

pub async fn ensure_bucket(client: &Client, bucket_name: &str) -> Result<(), anyhow::Error> {
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

pub async fn start_server(state: Arc<AppState>, port: &str) -> Result<(), anyhow::Error> {
    let addr = format!("0.0.0.0:{}", port);
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
        .with_state(state);

    info!("WebSocket endpoint available at ws://{}/[doc_id]", addr);
    info!(
        "HTTP API endpoints available at http://{}/api/document/...",
        addr
    );
    axum::serve(listener, app).await?;

    Ok(())
}

#[cfg(feature = "auth")]
async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(doc_id): Path<String>,
    Query(query): Query<AuthQuery>,
    State(state): State<ServerState>,
) -> Response<Body> {
    crate::ws::ws_handler(ws, Path(doc_id), Query(query), State(state.app_state)).await
}

#[cfg(not(feature = "auth"))]
async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(doc_id): Path<String>,
    State(state): State<ServerState>,
) -> Response<Body> {
    crate::ws::ws_handler(ws, Path(doc_id), State(state.app_state)).await
}
