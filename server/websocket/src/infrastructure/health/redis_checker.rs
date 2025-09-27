use crate::domain::entity::health::ComponentHealth;
use crate::domain::repository::health::{HealthCheckError, HealthChecker, RedisHealthChecker};
use crate::infrastructure::redis::RedisStore;
use async_trait::async_trait;
use std::sync::Arc;

pub struct RedisHealthCheckerImpl {
    redis_store: Arc<RedisStore>,
}

impl RedisHealthCheckerImpl {
    pub fn new(redis_store: Arc<RedisStore>) -> Self {
        Self { redis_store }
    }
}

#[async_trait]
impl HealthChecker for RedisHealthCheckerImpl {
    async fn check_health(&self) -> Result<ComponentHealth, HealthCheckError> {
        match self.ping().await {
            Ok(_) => Ok(ComponentHealth::healthy("Redis connection is working")),
            Err(e) => Ok(ComponentHealth::unhealthy(format!("Redis check failed: {}", e))),
        }
    }

    fn component_name(&self) -> &str {
        "redis"
    }
}

#[async_trait]
impl RedisHealthChecker for RedisHealthCheckerImpl {
    async fn ping(&self) -> Result<(), HealthCheckError> {
        match self.redis_store.get_pool().get().await {
            Ok(mut conn) => {
                match redis::cmd("PING").query_async::<String>(&mut *conn).await {
                    Ok(_) => Ok(()),
                    Err(e) => Err(HealthCheckError::Connection(format!("Redis PING failed: {}", e))),
                }
            }
            Err(e) => Err(HealthCheckError::Connection(format!("Failed to get Redis connection: {}", e))),
        }
    }
}
