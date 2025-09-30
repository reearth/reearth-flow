use std::sync::Arc;

use anyhow::{Context, Result};
use tracing::{error, info};
use uuid::Uuid;

use crate::application::usecases::document_usecase::DocumentUseCase;
use crate::domain::repositories::document::DocumentRepository;
use crate::infrastructure::gcs::GcsStore;
use crate::infrastructure::redis::RedisStore;
use crate::infrastructure::repository::document::DocumentRepositoryImpl;
use crate::infrastructure::websocket::{BroadcastPool, CollaborativeStorage};
use crate::presentation::http;
use crate::{conf::Config, AppState, WebsocketUseCase};

#[cfg(feature = "auth")]
use crate::auth::AuthService;

pub struct ApplicationContext {
    pub config: Config,
    pub state: Arc<AppState>,
}

pub async fn build() -> Result<ApplicationContext> {
    let config = Config::load().context("failed to load configuration")?;
    build_with_config(config).await
}

pub async fn build_with_config(config: Config) -> Result<ApplicationContext> {
    let gcs_store = GcsStore::new_with_config(config.gcs.clone())
        .await
        .context("failed to create GCS store")?;
    info!("GCS store initialized");
    let gcs_store = Arc::new(gcs_store);

    let redis_store = RedisStore::new(config.redis.clone())
        .await
        .context("failed to initialize Redis store")?;
    info!("Redis store initialized");
    let redis_store = Arc::new(redis_store);

    let collaborative_storage = Arc::new(CollaborativeStorage::new(
        Arc::clone(&gcs_store),
        Arc::clone(&redis_store),
    ));

    let pool = Arc::new(BroadcastPool::new(
        Arc::clone(&gcs_store),
        Arc::clone(&redis_store),
    ));

    let document_repository: Arc<dyn DocumentRepository> = Arc::new(DocumentRepositoryImpl::new(
        Arc::clone(&gcs_store),
        Arc::clone(&collaborative_storage),
    ));
    let document_usecase = Arc::new(DocumentUseCase::new(document_repository));
    let websocket_usecase = Arc::new(WebsocketUseCase::new(Arc::clone(&pool)));

    let instance_id = Uuid::new_v4().to_string();
    info!("Generated instance ID: {}", instance_id);

    let state = Arc::new({
        #[cfg(feature = "auth")]
        {
            let auth = AuthService::new(config.auth.clone())
                .await
                .context("failed to initialize auth service")?;
            info!("Auth service initialized");
            AppState {
                pool,
                document_usecase: Arc::clone(&document_usecase),
                websocket_usecase: Arc::clone(&websocket_usecase),
                auth: Arc::new(auth),
                instance_id,
            }
        }
        #[cfg(not(feature = "auth"))]
        {
            AppState {
                pool,
                document_usecase,
                websocket_usecase,
                instance_id,
            }
        }
    });

    Ok(ApplicationContext { config, state })
}

pub async fn run() -> Result<()> {
    let ApplicationContext { state, config } = build().await?;

    let result = http::server::start_server(state, &config.ws_port, &config).await;

    if let Err(err) = &result {
        error!("Server error: {}", err);
    }

    result
}

pub async fn run_with_config(config: Config) -> Result<()> {
    let ApplicationContext { state, config } = build_with_config(config).await?;
    http::server::start_server(state, &config.ws_port, &config).await
}
