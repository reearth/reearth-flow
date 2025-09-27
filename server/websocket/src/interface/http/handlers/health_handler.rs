use crate::application::services::health_service::HealthService;
use crate::domain::entity::health::HealthStatus;
use axum::{http::StatusCode, response::Json};
use serde_json::Value;
use std::sync::Arc;
use tracing::{debug, info};

pub struct HealthHandler {
    health_service: Arc<HealthService>,
}

impl HealthHandler {
    pub fn new(health_service: Arc<HealthService>) -> Self {
        Self { health_service }
    }

    pub async fn check_health(&self) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
        debug!("Health check endpoint called");

        let system_health = self.health_service.check_system_health().await;
        let json_response = serde_json::to_value(&system_health).map_err(|e| {
            let error_response = serde_json::json!({
                "error": "Failed to serialize health response",
                "details": e.to_string()
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;

        match system_health.status {
            HealthStatus::Healthy => {
                info!("Health check passed - all components healthy");
                Ok(Json(json_response))
            }
            HealthStatus::Unhealthy => {
                info!("Health check failed - some components unhealthy");
                Err((StatusCode::SERVICE_UNAVAILABLE, Json(json_response)))
            }
        }
    }
}

pub async fn health_check_handler(
    axum::extract::Extension(handler): axum::extract::Extension<Arc<HealthHandler>>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    handler.check_health().await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::services::health_service::HealthService;
    use crate::domain::entity::health::ComponentHealth;
    use crate::domain::repository::health::{HealthCheckError, HealthChecker};
    use async_trait::async_trait;

    struct MockHealthyChecker;

    #[async_trait]
    impl HealthChecker for MockHealthyChecker {
        async fn check_health(&self) -> Result<ComponentHealth, HealthCheckError> {
            Ok(ComponentHealth::healthy("Mock is healthy"))
        }

        fn component_name(&self) -> &str {
            "mock"
        }
    }

    #[tokio::test]
    async fn test_health_handler_healthy() {
        let mut service = HealthService::new("test");
        service.add_checker(Arc::new(MockHealthyChecker));
        let handler = HealthHandler::new(Arc::new(service));

        let result = handler.check_health().await;
        assert!(result.is_ok());
    }
}
