use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Unhealthy,
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "healthy"),
            HealthStatus::Unhealthy => write!(f, "unhealthy"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub status: HealthStatus,
    pub message: String,
    pub checked_at: DateTime<Utc>,
}

impl ComponentHealth {
    pub fn healthy(message: impl Into<String>) -> Self {
        Self {
            status: HealthStatus::Healthy,
            message: message.into(),
            checked_at: Utc::now(),
        }
    }

    pub fn unhealthy(message: impl Into<String>) -> Self {
        Self {
            status: HealthStatus::Unhealthy,
            message: message.into(),
            checked_at: Utc::now(),
        }
    }

    pub fn is_healthy(&self) -> bool {
        matches!(self.status, HealthStatus::Healthy)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    pub status: HealthStatus,
    pub service: String,
    pub timestamp: DateTime<Utc>,
    pub components: HashMap<String, ComponentHealth>,
}

impl SystemHealth {
    pub fn new(service: impl Into<String>) -> Self {
        Self {
            status: HealthStatus::Healthy,
            service: service.into(),
            timestamp: Utc::now(),
            components: HashMap::new(),
        }
    }

    pub fn add_component(&mut self, name: impl Into<String>, health: ComponentHealth) {
        self.components.insert(name.into(), health);
        self.update_overall_status();
    }

    pub fn is_healthy(&self) -> bool {
        matches!(self.status, HealthStatus::Healthy)
    }

    fn update_overall_status(&mut self) {
        let all_healthy = self
            .components
            .values()
            .all(|component| component.is_healthy());

        self.status = if all_healthy {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unhealthy
        };

        self.timestamp = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_health_creation() {
        let healthy = ComponentHealth::healthy("All good");
        assert!(healthy.is_healthy());
        assert_eq!(healthy.message, "All good");

        let unhealthy = ComponentHealth::unhealthy("Connection failed");
        assert!(!unhealthy.is_healthy());
        assert_eq!(unhealthy.message, "Connection failed");
    }

    #[test]
    fn test_system_health_aggregation() {
        let mut system = SystemHealth::new("websocket");
        assert!(system.is_healthy());

        // Add healthy component
        system.add_component("redis", ComponentHealth::healthy("Redis OK"));
        assert!(system.is_healthy());

        // Add unhealthy component
        system.add_component("gcs", ComponentHealth::unhealthy("GCS failed"));
        assert!(!system.is_healthy());
    }
}
