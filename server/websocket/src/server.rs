use axum::{routing::get, Router};
use google_cloud_storage::{
    client::Client,
    http::buckets::insert::{BucketCreationConfig, InsertBucketRequest},
};
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;

use crate::{ws::ws_handler, AppState};

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

    info!("Starting WebSocket server on {}", addr);

    let app = Router::new()
        .route("/{doc_id}", get(ws_handler))
        .with_state(state);

    axum::serve(listener, app).await?;

    Ok(())
}
