use crate::domain::entities::health::ComponentHealth;
use crate::domain::repositories::health::{GcsHealthChecker, HealthCheckError, HealthChecker};
use crate::infrastructure::gcs::GcsStore;
use async_trait::async_trait;
use std::sync::Arc;

pub struct GcsHealthCheckerImpl {
    gcs_store: Arc<GcsStore>,
}

impl GcsHealthCheckerImpl {
    pub fn new(gcs_store: Arc<GcsStore>) -> Self {
        Self { gcs_store }
    }
}

#[async_trait]
impl HealthChecker for GcsHealthCheckerImpl {
    async fn check_health(&self) -> Result<ComponentHealth, HealthCheckError> {
        match self.list_objects().await {
            Ok(_) => Ok(ComponentHealth::healthy("GCS connection is working")),
            Err(e) => Ok(ComponentHealth::unhealthy(
                format!("GCS check failed: {e}",),
            )),
        }
    }

    fn component_name(&self) -> &str {
        "gcs"
    }
}

#[async_trait]
impl GcsHealthChecker for GcsHealthCheckerImpl {
    async fn list_objects(&self) -> Result<(), HealthCheckError> {
        let request = google_cloud_storage::http::objects::list::ListObjectsRequest {
            bucket: self.gcs_store.bucket.clone(),
            max_results: Some(1),
            ..Default::default()
        };

        match self.gcs_store.client.list_objects(&request).await {
            Ok(_) => Ok(()),
            Err(e) => Err(HealthCheckError::Connection(format!(
                "GCS connection failed: {e}",
            ))),
        }
    }
}
