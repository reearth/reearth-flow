use std::sync::Arc;

use tracing::error;
use websocket::{
    conf::Config, pool::BroadcastPool, server::start_server, storage::gcs::GcsStore, AppState,
};

#[cfg(feature = "auth")]
use websocket::auth::AuthService;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_file(true)
        .with_line_number(true)
        .init();

    let config = match Config::load() {
        Ok(config) => config,
        Err(e) => {
            error!("Failed to load config: {}", e);
            std::process::exit(1);
        }
    };

    let store = match GcsStore::new_with_config(config.gcs.clone()).await {
        Ok(store) => store,
        Err(e) => {
            error!("Failed to create GCS store: {}", e);
            std::process::exit(1);
        }
    };

    // if let Err(e) = websocket::ensure_bucket(&store.client, &config.gcs.bucket_name).await {
    //     error!("Failed to ensure bucket exists: {}", e);
    //     std::process::exit(1);
    // }

    let store = Arc::new(store);
    tracing::info!("GCS store initialized");

    let pool = Arc::new(BroadcastPool::new(store, Some(config.redis)));
    tracing::info!("Broadcast pool initialized");

    let state = Arc::new({
        #[cfg(feature = "auth")]
        {
            let auth = match AuthService::new(config.auth).await {
                Ok(auth) => Arc::new(auth),
                Err(e) => {
                    error!("Failed to initialize auth service: {}", e);
                    std::process::exit(1);
                }
            };
            tracing::info!("Auth service initialized");
            AppState { pool, auth }
        }
        #[cfg(not(feature = "auth"))]
        {
            AppState { pool }
        }
    });

    if let Err(e) = start_server(state, &config.ws_port).await {
        error!("Server error: {}", e);
        std::process::exit(1);
    }
}
