use crate::project::{Project, ProjectEditingSession};
use crate::snapshot::ProjectSnapshot;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait ProjectRepository {
    async fn get_project(&self, project_id: &str) -> Result<Option<Project>>;
}

#[async_trait]
pub trait ProjectEditingSessionRepository {
    async fn create_session(&self, session: ProjectEditingSession) -> Result<()>;
    async fn get_active_session(&self, project_id: &str) -> Result<Option<ProjectEditingSession>>;
    async fn update_session(&self, session: ProjectEditingSession) -> Result<()>;
}

#[async_trait]
pub trait ProjectSnapshotRepository {
    async fn create_snapshot(&self, snapshot: ProjectSnapshot) -> Result<()>;
    async fn get_latest_snapshot(&self, project_id: &str) -> Result<Option<ProjectSnapshot>>;
    async fn get_latest_snapshot_state(&self, project_id: &str) -> Result<Vec<u8>>;
}
