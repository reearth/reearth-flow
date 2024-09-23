use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ObjectDelete {
    pub deleted: bool,
    pub delete_after: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ObjectTenant {
    pub id: String,
    pub key: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectMetadata {
    pub id: String,
    pub project_id: String,
    pub session_id: Option<String>,
    pub name: String,
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectSnapshot {
    pub metadata: ProjectMetadata,
    pub created_by: Option<String>,
    pub changes_by: Vec<String>,
    pub tenant: ObjectTenant,
    pub delete: ObjectDelete,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl ProjectSnapshot {
    pub fn new(
        metadata: ProjectMetadata,
        created_by: Option<String>,
        changes_by: Vec<String>,
        tenant: ObjectTenant,
        delete: ObjectDelete,
        created_at: Option<DateTime<Utc>>,
        updated_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            metadata,
            created_by,
            changes_by,
            tenant,
            delete,
            created_at,
            updated_at,
        }
    }
}
