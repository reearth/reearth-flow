use anyhow::Result;
use deadpool::Runtime;
use deadpool_redis::{Config, Pool};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct RedisConfig {
    pub url: String,
    pub ttl: u64,
}

#[derive(Debug, Clone)]
pub struct RedisStore {
    pool: Arc<Pool>,
    config: RedisConfig,
}

impl RedisStore {
    pub async fn new(config: RedisConfig) -> Result<Self> {
        let cfg = Config::from_url(&config.url);
        let pool = cfg.create_pool(Some(Runtime::Tokio1))?;
        let pool = Arc::new(pool);
        Ok(Self { pool, config })
    }

    pub fn get_pool(&self) -> Arc<Pool> {
        self.pool.clone()
    }

    pub fn get_config(&self) -> RedisConfig {
        self.config.clone()
    }

    pub async fn create_dedicated_connection(&self) -> Result<redis::aio::MultiplexedConnection> {
        let client = redis::Client::open(self.config.url.clone())?;
        let conn = client.get_multiplexed_async_connection().await?;
        Ok(conn)
    }
}
