use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone, Debug)]
pub struct ManageProjectEditSessionTaskData {
    pub project_id: String,
    pub last_merged_at: Arc<RwLock<Option<DateTime<Utc>>>>,
    pub last_snapshot_at: Arc<RwLock<Option<DateTime<Utc>>>>,
    pub clients_disconnected_at: Arc<RwLock<Option<DateTime<Utc>>>>,
    pub client_count: Arc<RwLock<Option<usize>>>,
}

impl ManageProjectEditSessionTaskData {
    pub fn new(
        project_id: String,
        last_merged_at: Option<DateTime<Utc>>,
        last_snapshot_at: Option<DateTime<Utc>>,
        clients_disconnected_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            project_id,
            last_merged_at: Arc::new(RwLock::new(last_merged_at)),
            last_snapshot_at: Arc::new(RwLock::new(last_snapshot_at)),
            clients_disconnected_at: Arc::new(RwLock::new(clients_disconnected_at)),
            client_count: Arc::new(RwLock::new(None)),
        }
    }
}
