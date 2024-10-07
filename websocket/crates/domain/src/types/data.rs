use core::time;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SnapshotData {
    pub project_id: String,
    pub state: Vec<u8>,
    pub name: Option<String>,
    pub created_by: Option<String>,
}

impl SnapshotData {
    pub fn new(
        project_id: String,
        state: Vec<u8>,
        name: Option<String>,
        created_by: Option<String>,
    ) -> Self {
        Self {
            project_id,
            state,
            name,
            created_by,
        }
    }
}
