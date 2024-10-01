use crate::project::ProjectEditingSession;
use crate::projection::Project;
use crate::types::snapshot::ProjectSnapshot;
use std::error::Error;

#[async_trait::async_trait]
pub trait ProjectRepository<E: Error + Send + Sync> {
    async fn get_project(&self, project_id: &str) -> Result<Option<Project>, E>;
}

#[async_trait::async_trait]
pub trait ProjectEditingSessionRepository<E: Error + Send + Sync> {
    async fn create_session(&self, session: ProjectEditingSession) -> Result<(), E>;
    async fn get_active_session(
        &self,
        project_id: &str,
    ) -> Result<Option<ProjectEditingSession>, E>;
    async fn update_session(&self, session: ProjectEditingSession) -> Result<(), E>;
}

#[async_trait::async_trait]
pub trait ProjectSnapshotRepository<E: Error + Send + Sync> {
    async fn create_snapshot(&self, snapshot: ProjectSnapshot) -> Result<(), E>;
    async fn update_latest_snapshot(&self, snapshot: ProjectSnapshot) -> Result<(), E>;
    async fn get_latest_snapshot(&self, project_id: &str) -> Result<Option<ProjectSnapshot>, E>;
    async fn get_latest_snapshot_state(&self, project_id: &str) -> Result<Vec<u8>, E>;
    async fn update_snapshot(&self, snapshot: ProjectSnapshot) -> Result<(), E>;
}
