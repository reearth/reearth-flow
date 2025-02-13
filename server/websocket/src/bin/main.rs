use std::sync::Arc;

use tracing::error;
use websocket::{
    conf::Config,
    pool::BroadcastPool,
    server::{create_router, ensure_bucket, start_server},
    storage::gcs::GcsStore,
    AppState,
};

#[cfg(feature = "auth")]
use websocket::auth::AuthService;

#[tokio::main]
async fn main() {
    // Initialize tracing
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

    let store = GcsStore::new_with_config(config.gcs)
        .await
        .expect("Failed to create GCS store");

    // Ensure bucket exists
    if let Err(e) = ensure_bucket(&store.client).await {
        error!("Failed to ensure bucket exists: {}", e);
        std::process::exit(1);
    }

    let store = Arc::new(store);
    tracing::info!("GCS store initialized");

    // Create broadcast pool
    let pool = Arc::new(BroadcastPool::new(store, config.redis));
    tracing::info!("Broadcast pool initialized");

    let state = {
        #[cfg(feature = "auth")]
        {
            let auth = Arc::new(AuthService::new(config.auth));
            tracing::info!("Auth service initialized");
            AppState { pool, auth }
        }
        #[cfg(not(feature = "auth"))]
        {
            AppState { pool }
        }
    };

    let app = create_router(state);

    if let Err(e) = start_server(app).await {
        error!("Server error: {}", e);
        std::process::exit(1);
    }
}
