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

    pub fn should_delete(&self) -> bool {
        if !self.deleted {
            return false;
        }

        if let Some(delete_after) = self.delete_after {
            Utc::now() >= delete_after
        } else {
            true
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
    pub version: u64,
    pub parent_version: Option<u64>,
    pub snapshot_type: SnapshotType,
    pub hash: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SnapshotType {
    Auto,
    Manual,
    Checkpoint,
    Recovery,
}

impl ProjectSnapshot {
    pub fn new(
        project_id: String,
        created_by: String,
        data: Vec<u8>,
        snapshot_type: SnapshotType,
        parent_version: Option<u64>,
        version: u64,
        path: String,
    ) -> Self {
        let now = Utc::now();
        let snapshot_id = generate_id!("snapshot");

        Self {
            metadata: Metadata::new(
                snapshot_id.clone(),
                project_id,
                Some(generate_id!("session")),
                None,
                path,
            ),
            info: SnapshotInfo::new(
                created_by,
                vec![],
                ObjectTenant::default(),
                ObjectDelete::new(false, None),
                Some(now),
                Some(now),
            ),
            data: data.clone(),
            version,
            parent_version,
            snapshot_type,
            hash: Self::calculate_hash(&data),
        }
    }

    fn calculate_hash(data: &[u8]) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    pub fn verify_integrity(&self) -> bool {
        self.hash == Self::calculate_hash(&self.data)
    }

    pub fn create_diff_snapshot(
        &self,
        new_data: Vec<u8>,
        created_by: String,
        snapshot_type: SnapshotType,
    ) -> Self {
        let now = Utc::now();
        let snapshot_id = generate_id!("snapshot");

        Self {
            metadata: Metadata::new(
                snapshot_id,
                self.metadata.project_id.clone(),
                Some(generate_id!("session")),
                None,
                self.metadata.path.clone(),
            ),
            info: SnapshotInfo::new(
                created_by,
                vec![],
                self.info.tenant.clone(),
                ObjectDelete::new(false, None),
                Some(now),
                Some(now),
            ),
            data: new_data.clone(),
            version: self.version + 1,
            parent_version: Some(self.version),
            snapshot_type,
            hash: Self::calculate_hash(&new_data),
        }
    }

    pub fn get_metadata(&self) -> &Metadata {
        &self.metadata
    }

    pub fn get_created_at(&self) -> Option<DateTime<Utc>> {
        self.info.created_at
    }

    pub fn get_updated_at(&self) -> Option<DateTime<Utc>> {
        self.info.updated_at
    }

    pub fn is_deleted(&self) -> bool {
        self.info.delete.deleted
    }

    pub fn mark_as_deleted(&mut self, delete_after: Option<DateTime<Utc>>) {
        self.info.delete = ObjectDelete::new(true, delete_after);
        self.info.updated_at = Some(Utc::now());
    }

    pub fn add_collaborator(&mut self, user_id: String) {
        if !self.info.changes_by.contains(&user_id) {
            self.info.changes_by.push(user_id);
            self.info.updated_at = Some(Utc::now());
        }
    }

    pub fn builder() -> ProjectSnapshotBuilder {
        ProjectSnapshotBuilder::default()
    }
}

#[derive(Debug, Default)]
pub struct ProjectSnapshotBuilder {
    project_id: Option<String>,
    created_by: Option<String>,
    data: Option<Vec<u8>>,
    snapshot_type: Option<SnapshotType>,
    parent_version: Option<u64>,
    version: Option<u64>,
    path: Option<String>,
    tenant_id: Option<String>,
    metadata: Option<SnapshotMetadata>,
    session_id: Option<String>,
    name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SnapshotMetadata {
    pub name: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub custom_properties: std::collections::HashMap<String, String>,
}

impl ProjectSnapshotBuilder {
    pub fn project_id(mut self, project_id: String) -> Self {
        self.project_id = Some(project_id);
        self
    }

    pub fn created_by(mut self, created_by: String) -> Self {
        self.created_by = Some(created_by);
        self
    }

    pub fn data(mut self, data: Vec<u8>) -> Self {
        self.data = Some(data);
        self
    }

    pub fn snapshot_type(mut self, snapshot_type: SnapshotType) -> Self {
        self.snapshot_type = Some(snapshot_type);
        self
    }

    pub fn version(mut self, version: u64) -> Self {
        self.version = Some(version);
        self
    }

    pub fn parent_version(mut self, parent_version: Option<u64>) -> Self {
        self.parent_version = parent_version;
        self
    }

    pub fn path(mut self, path: String) -> Self {
        self.path = Some(path);
        self
    }

    pub fn tenant_id(mut self, tenant_id: String) -> Self {
        self.tenant_id = Some(tenant_id);
        self
    }

    pub fn metadata(mut self, metadata: SnapshotMetadata) -> Self {
        self.metadata = Some(metadata);
        self
    }

    pub fn session_id(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }

    pub fn name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn build(self) -> Result<ProjectSnapshot, &'static str> {
        let project_id = self.project_id.ok_or("project_id is required")?;
        let created_by = self.created_by.ok_or("created_by is required")?;
        let data = self.data.ok_or("data is required")?;
        let snapshot_type = self.snapshot_type.unwrap_or(SnapshotType::Manual);
        let version = self.version.ok_or("version is required")?;
        let path = self.path.unwrap_or_default();

        let now = Utc::now();
        let snapshot_id = generate_id!("snapshot");
        let tenant = ObjectTenant::new(
            self.tenant_id.unwrap_or_else(|| generate_id!("tenant")),
            "".to_string(),
        );

        let metadata = Metadata::new(
            snapshot_id.clone(),
            project_id,
            self.session_id,
            self.name,
            path,
        );

        let info = SnapshotInfo::new(
            created_by,
            vec![],
            tenant,
            ObjectDelete::new(false, None),
            Some(now),
            Some(now),
        );

        Ok(ProjectSnapshot {
            metadata,
            info,
            data: data.clone(),
            version,
            parent_version: self.parent_version,
            snapshot_type,
            hash: ProjectSnapshot::calculate_hash(&data),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_creation() {
        let data = b"test data".to_vec();
        let snapshot = ProjectSnapshot::new(
            "project1".to_string(),
            "user1".to_string(),
            data.clone(),
            SnapshotType::Manual,
            None,
            1,
            "/path/to/file".to_string(),
        );

        assert!(snapshot.metadata.id.starts_with("snapshot"));
        assert_eq!(snapshot.version, 1);
        assert!(snapshot.verify_integrity());
    }

    #[test]
    fn test_diff_snapshot() {
        let original_data = b"original data".to_vec();
        let original = ProjectSnapshot::new(
            "project1".to_string(),
            "user1".to_string(),
            original_data,
            SnapshotType::Manual,
            None,
            1,
            "/path/to/file".to_string(),
        );

        let new_data = b"updated data".to_vec();
        let diff = original.create_diff_snapshot(new_data, "user2".to_string(), SnapshotType::Auto);

        assert_eq!(diff.version, 2);
        assert_eq!(diff.parent_version, Some(1));
        assert!(diff.verify_integrity());
    }

    #[test]
    fn test_snapshot_builder() {
        let data = b"test data".to_vec();
        let snapshot = ProjectSnapshot::builder()
            .project_id("project1".to_string())
            .created_by("user1".to_string())
            .data(data.clone())
            .snapshot_type(SnapshotType::Manual)
            .version(1)
            .path("/path/to/file".to_string())
            .build()
            .unwrap();

        assert!(snapshot.metadata.id.starts_with("snapshot"));
        assert_eq!(snapshot.version, 1);
        assert!(snapshot.verify_integrity());
    }
}
