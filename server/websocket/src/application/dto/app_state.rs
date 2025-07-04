use crate::application::services::DocumentAppService;
use crate::infrastructure::BroadcastPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub pool: Arc<BroadcastPool>,
    pub document_service: Arc<DocumentAppService>,
    pub instance_id: String,
}

impl AppState {
    pub fn new(
        pool: Arc<BroadcastPool>,
        document_service: Arc<DocumentAppService>,
        instance_id: String,
    ) -> Self {
        Self {
            pool,
            document_service,
            instance_id,
        }
    }
}
