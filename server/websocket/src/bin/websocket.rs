use std::sync::Arc;

use tracing::error;
use uuid::Uuid;
use websocket::{
    conf::Config, infrastructure::gcs::GcsStore,
    infrastructure::redis::stream_trimmer::spawn_stream_trimmer, infrastructure::redis::RedisStore,
    pool::BroadcastPool, server::start_server, AppState,
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

    let redis_store = match RedisStore::new(config.redis.clone()).await {
        Ok(redis_store) => {
            tracing::info!("Redis store initialized");
            Some(Arc::new(redis_store))
        }
        Err(e) => {
            error!("Failed to initialize Redis store: {}", e);
            None
        }
    };

    let pool = Arc::new(match &redis_store {
        Some(rs) => BroadcastPool::new(Arc::clone(&store), Arc::clone(rs)),
        None => {
            error!("Cannot proceed without Redis store");
            std::process::exit(1);
        }
    });

    let trimmer_shutdown = if let Some(ref rs) = redis_store {
        let trimmer_shutdown = spawn_stream_trimmer(
            Arc::clone(&pool),
            Arc::clone(rs),
            Arc::clone(&store),
            config.redis.stream_trim_interval,
            config.redis.stream_max_message_age,
            config.redis.stream_max_length,
        );
        tracing::info!(
            "Stream trimmer started with awareness integration: interval: {}s, max age: {}ms, max length: {}",
            config.redis.stream_trim_interval,
            config.redis.stream_max_message_age,
            config.redis.stream_max_length
        );
        Some(trimmer_shutdown)
    } else {
        None
    };

    let instance_id = Uuid::new_v4().to_string();
    tracing::info!("Generated instance ID: {}", instance_id);

    let state = Arc::new({
        #[cfg(feature = "auth")]
        {
            let auth = match AuthService::new(config.auth.clone()).await {
                Ok(auth) => Arc::new(auth),
                Err(e) => {
                    error!("Failed to initialize auth service: {}", e);
                    std::process::exit(1);
                }
            };
            tracing::info!("Auth service initialized");
            AppState {
                pool,
                auth,
                instance_id,
            }
        }
        #[cfg(not(feature = "auth"))]
        {
            AppState { pool, instance_id }
        }
    });

    let server_result = start_server(state, &config.ws_port, &config).await;

    if let Some(shutdown_sender) = trimmer_shutdown {
        if shutdown_sender.send(()).is_err() {
            tracing::warn!("Stream trimmer already stopped");
        } else {
            tracing::info!("Sent shutdown signal to stream trimmer");
        }
    }

    if let Err(e) = server_result {
        error!("Server error: {}", e);
        std::process::exit(1);
    }
}
