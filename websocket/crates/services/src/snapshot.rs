use async_trait::async_trait;
use chrono::Utc;
use flow_websocket_domain::snapshot::{ProjectSnapshot, ProjectMetadata, ObjectDelete, ObjectTenant};
use flow_websocket_domain::repository::ProjectSnapshotRepository;
use std::sync::Arc;
use std::error::Error;
use uuid::Uuid;

pub struct SnapshotService {
    snapshot_repository: Arc<dyn ProjectSnapshotRepository>,
}

impl SnapshotService {
    pub fn new(snapshot_repository: Arc<dyn ProjectSnapshotRepository>) -> Self {
        Self {
            snapshot_repository,
        }
    }

    pub async fn create_snapshot(&self, data: CreateSnapshotData) -> Result<ProjectSnapshot, Box<dyn Error>> {
        let snapshot_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let snapshot = ProjectSnapshot {
            metadata: ProjectMetadata {
                id: snapshot_id.clone(),
                project_id: data.project_id.clone(),
                session_id: data.session_id,
                name: data.name.unwrap_or_default(),
                path: String::new(),
            },
            created_by: data.created_by,
            changes_by: data.changes_by,
            tenant: data.tenant,
            delete: ObjectDelete {
                deleted: false,
                delete_after: None,
            },
            created_at: Some(now),
            updated_at: Some(now),
        };

        self.snapshot_repository.create_snapshot(snapshot.clone()).await?;

        Ok(snapshot)
    }

    pub async fn get_latest_snapshot(&self, project_id: &str) -> Result<Option<ProjectSnapshot>, Box<dyn Error>> {
        self.snapshot_repository.get_latest_snapshot(project_id).await
    }

    pub async fn get_latest_snapshot_state(&self, project_id: &str) -> Result<Vec<u8>, Box<dyn Error>> {
        self.snapshot_repository.get_latest_snapshot_state(project_id).await
    }
}

#[async_trait]
impl ProjectSnapshotRepository for SnapshotService {
    async fn create_snapshot(&self, snapshot: ProjectSnapshot) -> Result<(), Box<dyn Error>> {
        self.snapshot_repository.create_snapshot(snapshot).await
    }

    async fn get_latest_snapshot(&self, project_id: &str) -> Result<Option<ProjectSnapshot>, Box<dyn Error>> {
        self.snapshot_repository.get_latest_snapshot(project_id).await
    }

    async fn get_latest_snapshot_state(&self, project_id: &str) -> Result<Vec<u8>, Box<dyn Error>> {
        self.snapshot_repository.get_latest_snapshot_state(project_id).await
    }
}

pub struct CreateSnapshotData {
    pub project_id: String,
    pub session_id: Option<String>,
    pub name: Option<String>,
    pub created_by: Option<String>,
    pub changes_by: Vec<String>,
    pub tenant: ObjectTenant,
    pub state: Vec<u8>,
}
