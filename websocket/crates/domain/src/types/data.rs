use serde::{Deserialize, Serialize};

use super::snapshot::ObjectTenant;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SnapshotData {
    pub project_id: String,
    pub session_id: Option<String>,
    pub name: Option<String>,
    pub created_by: Option<String>,
    pub changes_by: Vec<String>,
    pub tenant: ObjectTenant,
    pub state: Vec<u8>,
}
