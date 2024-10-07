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

impl Default for ObjectTenant {
    fn default() -> Self {
        Self {
            id: "".to_string(),
            key: "".to_string(),
        }
    }
}

impl ObjectTenant {
    pub fn new(id: String, key: String) -> Self {
        Self { id, key }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Metadata {
    pub id: String,
    pub project_id: String,
    pub session_id: Option<String>,
    pub name: String,
    pub path: String,
}

impl Metadata {
    pub fn new(
        id: String,
        project_id: String,
        session_id: Option<String>,
        name: String,
        path: String,
    ) -> Self {
        Self {
            id,
            project_id,
            session_id,
            name,
            path,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SnapshotInfo {
    pub created_by: Option<String>,
    pub changes_by: Vec<String>,
    pub tenant: ObjectTenant,
    pub delete: ObjectDelete,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl SnapshotInfo {
    pub fn new(
        created_by: Option<String>,
        changes_by: Vec<String>,
        tenant: ObjectTenant,
        delete: ObjectDelete,
        created_at: Option<DateTime<Utc>>,
        updated_at: Option<DateTime<Utc>>,
    ) -> Self {
        let now = Utc::now();
        Self {
            created_by,
            changes_by,
            tenant,
            delete,
            created_at: created_at.or(Some(now)),
            updated_at: updated_at.or(Some(now)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectSnapshot {
    pub metadata: Metadata,
    pub info: SnapshotInfo,
}

impl ProjectSnapshot {
    pub fn new(metadata: Metadata, info: SnapshotInfo) -> Self {
        Self { metadata, info }
    }
}
