use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::generate_id;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ObjectDelete {
    pub deleted: bool,
    pub delete_after: Option<DateTime<Utc>>,
}

impl ObjectDelete {
    pub fn new(deleted: bool, delete_after: Option<DateTime<Utc>>) -> Self {
        Self {
            deleted,
            delete_after,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ObjectTenant {
    pub id: String,
    // user id
    pub key: String,
}

impl Default for ObjectTenant {
    fn default() -> Self {
        Self {
            id: generate_id!("tenant"),
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
    pub name: Option<String>,
    pub path: String,
}

impl Metadata {
    pub fn new(
        id: String,
        project_id: String,
        session_id: Option<String>,
        name: Option<String>,
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
    pub created_by: String,
    pub changes_by: Vec<String>,
    pub tenant: ObjectTenant,
    pub delete: ObjectDelete,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl SnapshotInfo {
    pub fn new(
        created_by: String,
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
    pub data: Vec<u8>,
}

impl ProjectSnapshot {
    pub fn new(metadata: Metadata, info: SnapshotInfo, data: Vec<u8>) -> Self {
        Self {
            metadata,
            info,
            data,
        }
    }
}
