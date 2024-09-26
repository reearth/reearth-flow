use async_trait::async_trait;
use chrono::Utc;
use flow_websocket_domain::repository::ProjectSnapshotRepository;
use flow_websocket_domain::snapshot::{
    Metadata, ObjectDelete, ObjectTenant, ProjectSnapshot, SnapshotInfo,
};
use std::error::Error;
use std::sync::Arc;
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

    pub async fn create_snapshot(
        &self,
        data: CreateSnapshotData,
    ) -> Result<ProjectSnapshot, Box<dyn Error>> {
        let snapshot_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let metadata = Metadata::new(
            snapshot_id.clone(),
            data.project_id.clone(),
            data.session_id,
            data.name.unwrap_or_default(),
            String::new(),
        );

        let state = SnapshotInfo::new(
            data.created_by,
            data.changes_by,
            data.tenant,
            ObjectDelete {
                deleted: false,
                delete_after: None,
            },
            Some(now),
            Some(now),
        );

        let snapshot = ProjectSnapshot::new(metadata, state);

        self.snapshot_repository
            .create_snapshot(snapshot.clone())
            .await?;

        Ok(snapshot)
    }

    pub async fn get_latest_snapshot(
        &self,
        project_id: &str,
    ) -> Result<Option<ProjectSnapshot>, Box<dyn Error>> {
        self.snapshot_repository
            .get_latest_snapshot(project_id)
            .await
    }

    pub async fn get_latest_snapshot_state(
        &self,
        project_id: &str,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        self.snapshot_repository
            .get_latest_snapshot_state(project_id)
            .await
    }
}

#[async_trait]
impl ProjectSnapshotRepository for SnapshotService {
    async fn create_snapshot(&self, snapshot: ProjectSnapshot) -> Result<(), Box<dyn Error>> {
        self.snapshot_repository.create_snapshot(snapshot).await
    }

    async fn get_latest_snapshot(
        &self,
        project_id: &str,
    ) -> Result<Option<ProjectSnapshot>, Box<dyn Error>> {
        self.snapshot_repository
            .get_latest_snapshot(project_id)
            .await
    }

    async fn get_latest_snapshot_state(&self, project_id: &str) -> Result<Vec<u8>, Box<dyn Error>> {
        self.snapshot_repository
            .get_latest_snapshot_state(project_id)
            .await
    }

    async fn update_snapshot(&self, snapshot: ProjectSnapshot) -> Result<(), Box<dyn Error>> {
        self.snapshot_repository.update_snapshot(snapshot).await
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

impl CreateSnapshotData {
    pub fn new(
        project_id: String,
        session_id: Option<String>,
        name: Option<String>,
        created_by: Option<String>,
        changes_by: Vec<String>,
        tenant: ObjectTenant,
        state: Vec<u8>,
    ) -> Self {
        Self {
            project_id,
            session_id,
            name,
            created_by,
            changes_by,
            tenant,
            state,
        }
    }
}
