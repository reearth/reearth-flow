use axum::Router;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::cors::{Any, CorsLayer};

use anyhow::Result;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;

use crate::application::{AppState, Config};
use crate::interface::{create_ws_router, document_routes};

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

pub async fn start_server(state: Arc<AppState>, port: &str, config: &Config) -> Result<()> {
    let addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(&addr).await?;

    info!("Starting server on {}", addr);

    let ws_router = create_ws_router(state.clone());

    let app = Router::new()
        .merge(ws_router)
        .nest("/api", document_routes().with_state(state.clone()))
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
