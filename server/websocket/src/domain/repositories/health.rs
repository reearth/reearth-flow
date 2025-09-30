use crate::domain::entities::health::ComponentHealth;
use async_trait::async_trait;

#[derive(Debug, thiserror::Error)]
pub enum HealthCheckError {
    #[error("Connection error: {0}")]
    Connection(String),
    #[error("Timeout error: {0}")]
    Timeout(String),
    #[error("Authentication error: {0}")]
    Authentication(String),
    #[error("Unknown error: {0}")]
    Unknown(String),
}

#[async_trait]
pub trait HealthChecker: Send + Sync {
    async fn check_health(&self) -> Result<ComponentHealth, HealthCheckError>;
    fn component_name(&self) -> &str;
}

#[async_trait]
pub trait RedisHealthChecker: HealthChecker {
    async fn ping(&self) -> Result<(), HealthCheckError>;
}

#[async_trait]
pub trait GcsHealthChecker: HealthChecker {
    async fn list_objects(&self) -> Result<(), HealthCheckError>;
}
