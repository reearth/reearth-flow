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
    pub session_id: String,
    pub project_id: String,
    pub clients_count: Option<usize>,
    pub last_merged_at: Option<DateTime<Utc>>,
    pub last_snapshot_at: Option<DateTime<Utc>>,
    pub clients_disconnected_at: Option<DateTime<Utc>>,
}
