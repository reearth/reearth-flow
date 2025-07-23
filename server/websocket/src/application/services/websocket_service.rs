use anyhow::Result;
use std::sync::Arc;
use uuid::Uuid;

use crate::application::dto::AppState;
use crate::application::services::{Config, DocumentAppService};
use crate::application::services::DocumentService;
use crate::infrastructure::repositories::DocumentRepositoryImpl;
use crate::infrastructure::{BroadcastPool, GcsStore, RedisStore};

pub struct WebSocketService;

impl WebSocketService {
    pub async fn initialize_app_state(config: &Config) -> Result<Arc<AppState>> {
        let gcs_store = GcsStore::new_with_config(config.gcs.clone().into()).await?;
        let gcs_store = Arc::new(gcs_store);
        tracing::info!("GCS store initialized");

        let redis_store = RedisStore::new(config.redis.clone().into()).await?;
        let redis_store = Arc::new(redis_store);
        tracing::info!("Redis store initialized");

        let pool = Arc::new(BroadcastPool::new(gcs_store, redis_store));
        tracing::info!("Broadcast pool initialized");

        let document_repository = Arc::new(DocumentRepositoryImpl::new(pool.clone()));

        let document_domain_service = Arc::new(DocumentService::new(document_repository));

        let document_app_service = Arc::new(DocumentAppService::new(
            document_domain_service,
            pool.clone(),
        ));

        let instance_id = Uuid::new_v4().to_string();
        tracing::info!("Generated instance ID: {}", instance_id);

        let state = Arc::new(AppState::new(pool, document_app_service, instance_id));

        Ok(state)
    }
}
