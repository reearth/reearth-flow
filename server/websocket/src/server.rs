use axum::{routing::get, Router};
use google_cloud_storage::{
    client::Client,
    http::buckets::insert::{BucketCreationConfig, InsertBucketRequest},
};
use tokio::net::TcpListener;
use tracing::info;

use crate::{
    handlers::{get_doc_history, get_latest_doc, rollback_doc, ws_handler},
    AppState, BUCKET_NAME, PORT,
};

pub async fn ensure_bucket(client: &Client) -> Result<(), anyhow::Error> {
    let bucket = BucketCreationConfig {
        location: "US".to_string(),
        ..Default::default()
    };
    let request = InsertBucketRequest {
        name: BUCKET_NAME.to_string(),
        bucket,
        ..Default::default()
    };

    match client.insert_bucket(&request).await {
        Ok(_) => Ok(()),
        Err(e) if e.to_string().contains("already exists") => Ok(()),
        Err(e) => Err(e.into()),
    }
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/{doc_id}", get(ws_handler))
        .route("/{doc_id}/latest", get(get_latest_doc))
        .route("/{doc_id}/history", get(get_doc_history))
        .route("/{doc_id}/rollback", get(rollback_doc))
        .with_state(state)
}

pub async fn setup_signal_handler() -> tokio::sync::broadcast::Sender<()> {
    let (tx, _) = tokio::sync::broadcast::channel(1);
    let shutdown_signal = tx.clone();

    tokio::spawn(async move {
        let ctrl_c = async {
            tokio::signal::ctrl_c()
                .await
                .expect("Failed to install Ctrl+C handler");
        };

        #[cfg(unix)]
        let terminate = async {
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .expect("Failed to install signal handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {},
            _ = terminate => {},
        }

        info!("Shutdown signal received");
        let _ = shutdown_signal.send(());
    });

    tx
}

pub async fn start_server(app: Router) -> Result<(), anyhow::Error> {
    info!("Starting server on 0.0.0.0:{}", PORT);
    let listener = TcpListener::bind(format!("0.0.0.0:{}", PORT)).await?;

    let tx = setup_signal_handler().await;

    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            let _ = tx.subscribe().recv().await;
        })
        .await?;

    Ok(())
}
