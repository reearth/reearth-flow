use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SnapshotData {
    pub project_id: String,
    pub state: Vec<u8>,
    pub name: Option<String>,
    pub created_by: Option<String>,
}
