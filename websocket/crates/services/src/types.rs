use chrono::{DateTime, Utc};
use flow_websocket_domain::snapshot::ObjectTenant;

#[derive(Clone, Debug)]
pub struct CreateSnapshotData {
    pub project_id: String,
    pub session_id: Option<String>,
    pub name: Option<String>,
    pub created_by: Option<String>,
    pub changes_by: Vec<String>,
    pub tenant: ObjectTenant,
    pub state: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct ManageProjectEditSessionTaskData {
    pub project_id: Option<String>,
    pub last_merged_at: Option<DateTime<Utc>>,
    pub last_snapshot_at: Option<DateTime<Utc>>,
    pub clients_disconnected_at: Option<DateTime<Utc>>,
}

impl ManageProjectEditSessionTaskData {
    pub fn new(
        project_id: Option<String>,
        last_merged_at: Option<DateTime<Utc>>,
        last_snapshot_at: Option<DateTime<Utc>>,
        clients_disconnected_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            project_id,
            last_merged_at,
            last_snapshot_at,
            clients_disconnected_at,
        }
    }
}

impl Default for ManageProjectEditSessionTaskData {
    fn default() -> Self {
        Self::new(None, None, None, None)
    }
}
