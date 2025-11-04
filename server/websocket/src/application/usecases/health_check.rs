use crate::domain::entities::health::SystemHealth;
use crate::domain::repositories::health::HealthChecker;
use std::sync::Arc;
use tracing::{debug, warn};

pub struct HealthCheckUseCase {
    checkers: Vec<Arc<dyn HealthChecker>>,
    service_name: String,
}

impl HealthCheckUseCase {
    pub fn new(service_name: impl Into<String>) -> Self {
        Self {
            checkers: Vec::new(),
            service_name: service_name.into(),
        }
    }

    pub fn add_checker(&mut self, checker: Arc<dyn HealthChecker>) {
        self.checkers.push(checker);
    }

    pub async fn check_system_health(&self) -> SystemHealth {
        let mut system_health = SystemHealth::new(&self.service_name);

        debug!(
            "Starting system health check with {} components",
            self.checkers.len()
        );

        for checker in &self.checkers {
            let component_name = checker.component_name();
            debug!("Checking health for component: {}", component_name);

            match checker.check_health().await {
                Ok(component_health) => {
                    debug!(
                        "Component {} health check completed: {:?}",
                        component_name, component_health.status
                    );
                    system_health.add_component(component_name, component_health);
                }
                Err(e) => {
                    warn!("Component {} health check failed: {}", component_name, e);
                    system_health.add_component(
                        component_name,
                        crate::domain::entities::health::ComponentHealth::unhealthy(format!(
                            "Health check failed: {e}",
                        )),
                    );
                }
            }
        }

        debug!(
            "System health check completed. Overall status: {:?}",
            system_health.status
        );
        system_health
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::health::ComponentHealth;
    use crate::domain::repositories::health::{HealthCheckError, HealthChecker};
    use async_trait::async_trait;

    struct MockHealthyChecker;
    struct MockUnhealthyChecker;

    #[async_trait]
    impl HealthChecker for MockHealthyChecker {
        async fn check_health(&self) -> Result<ComponentHealth, HealthCheckError> {
            Ok(ComponentHealth::healthy("Mock component is healthy"))
        }

        fn component_name(&self) -> &str {
            "mock_healthy"
        }
    }

    #[async_trait]
    impl HealthChecker for MockUnhealthyChecker {
        async fn check_health(&self) -> Result<ComponentHealth, HealthCheckError> {
            Ok(ComponentHealth::unhealthy("Mock component is unhealthy"))
        }

        fn component_name(&self) -> &str {
            "mock_unhealthy"
        }
    }

    #[tokio::test]
    async fn test_health_service_all_healthy() {
        let mut usecase = HealthCheckUseCase::new("test-service");
        usecase.add_checker(Arc::new(MockHealthyChecker));

        let health = usecase.check_system_health().await;
        assert!(health.is_healthy());
        assert_eq!(health.components.len(), 1);
    }

    #[tokio::test]
    async fn test_health_service_mixed_health() {
        let mut usecase = HealthCheckUseCase::new("test-service");
        usecase.add_checker(Arc::new(MockHealthyChecker));
        usecase.add_checker(Arc::new(MockUnhealthyChecker));

        let health = usecase.check_system_health().await;
        assert!(!health.is_healthy());
        assert_eq!(health.components.len(), 2);
    }
}
