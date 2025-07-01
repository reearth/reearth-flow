use anyhow::Result;
use std::sync::Arc;
use uuid::Uuid;

use crate::application::dto::AppState;
use crate::application::services::Config;
use crate::infrastructure::{BroadcastPool, GcsStore, RedisStore};

pub struct WebSocketService;

impl WebSocketService {
    /// 初始化WebSocket服务所需的应用状态
    pub async fn initialize_app_state(config: &Config) -> Result<Arc<AppState>> {
        // 初始化GCS存储
        let gcs_store = GcsStore::new_with_config(config.gcs.clone().into()).await?;
        let gcs_store = Arc::new(gcs_store);
        tracing::info!("GCS store initialized");

        // 初始化Redis存储
        let redis_store = RedisStore::new(config.redis.clone().into()).await?;
        let redis_store = Arc::new(redis_store);
        tracing::info!("Redis store initialized");

        // 创建广播池
        let pool = Arc::new(BroadcastPool::new(gcs_store, redis_store));
        tracing::info!("Broadcast pool initialized");

        // 生成实例ID
        let instance_id = Uuid::new_v4().to_string();
        tracing::info!("Generated instance ID: {}", instance_id);

        // 创建应用状态
        let state = Arc::new(AppState::new(pool, instance_id));

        Ok(state)
    }
}
