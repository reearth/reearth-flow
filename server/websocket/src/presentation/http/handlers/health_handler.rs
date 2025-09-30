use crate::application::usecases::health_check_usecase::HealthCheckUseCase;
use crate::domain::entities::health::HealthStatus;
use axum::{http::StatusCode, response::Json};
use serde_json::Value;
use std::sync::Arc;
use tracing::{debug, info};

pub struct HealthHandler {
    health_usecase: Arc<HealthCheckUseCase>,
}

impl HealthHandler {
    pub fn new(health_usecase: Arc<HealthCheckUseCase>) -> Self {
        Self { health_usecase }
    }

    pub async fn check_health(&self) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
        debug!("Health check endpoint called");

        let system_health = self.health_usecase.check_system_health().await;
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
    use crate::application::usecases::health_check_usecase::HealthCheckUseCase;
    use crate::domain::entities::health::ComponentHealth;
    use crate::domain::repositories::health::{HealthCheckError, HealthChecker};
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
        let mut usecase = HealthCheckUseCase::new("test");
        usecase.add_checker(Arc::new(MockHealthyChecker));
        let handler = HealthHandler::new(Arc::new(usecase));

        let result = handler.check_health().await;
        assert!(result.is_ok());
    }
}
