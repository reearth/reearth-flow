use std::sync::Arc;

use tracing::{debug, error, info};
use uuid::Uuid;
use websocket::{
    api::Api, conf::Config, pool::BroadcastPool, server::start_server, storage::gcs::GcsStore,
    storage::redis::RedisStore, subscriber::create_subscriber, worker::create_worker, AppState,
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

    let pool = Arc::new(match redis_store.clone() {
        Some(rs) => BroadcastPool::new(store.clone(), rs),
        None => {
            error!("Cannot proceed without Redis store");
            std::process::exit(1);
        }
    });

    let instance_id = Uuid::new_v4().to_string();
    info!("Generated instance ID: {}", instance_id);

    let api = match Api::new(redis_store.clone().unwrap(), store.clone(), None).await {
        Ok(api) => {
            info!("API initialized");
            Arc::new(api)
        }
        Err(e) => {
            error!("Failed to initialize API: {}", e);
            std::process::exit(1);
        }
    };

    let subscriber = match create_subscriber(redis_store.clone().unwrap(), api.clone()).await {
        Ok(subscriber) => Arc::new(subscriber),
        Err(e) => {
            error!("Failed to initialize Subscriber: {}", e);
            std::process::exit(1);
        }
    };

    let worker = match create_worker(api.clone(), None).await {
        Ok(worker) => Arc::new(worker),
        Err(e) => {
            error!("Failed to initialize Worker: {}", e);
            std::process::exit(1);
        }
    };

    // Start the worker in the background
    if let Err(e) = worker.start().await {
        error!("Failed to start worker: {}", e);
        std::process::exit(1);
    }
    info!("Worker started successfully");

    // Set up periodic cleanup for inactive broadcast groups
    let pool_for_cleanup = pool.clone();
    tokio::spawn(async move {
        let mut cleanup_interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
        cleanup_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            cleanup_interval.tick().await;
            debug!("Running periodic cleanup of inactive broadcast groups");
            pool_for_cleanup.cleanup_inactive_groups().await;
        }
    });

    // Set up graceful shutdown handlers
    let subscriber_clone = subscriber.clone();
    let worker_clone = worker.clone();
    let api_clone = api.clone();

    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            info!("Shutdown signal received, cleaning up...");

            // Stop worker
            if let Err(e) = worker_clone.stop().await {
                error!("Error stopping worker: {}", e);
            }

            // Destroy subscriber
            subscriber_clone.destroy().await;

            // Destroy API
            if let Err(e) = api_clone.destroy().await {
                error!("Error destroying API: {}", e);
            }

            info!("Cleanup completed");
        }
    });

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

    if let Err(e) = start_server(state, &config.ws_port, &config).await {
        error!("Server error: {}", e);
        std::process::exit(1);
    }
}
