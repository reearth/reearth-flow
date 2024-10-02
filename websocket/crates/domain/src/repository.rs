use crate::project::ProjectEditingSession;
use crate::projection::Project;
use crate::types::snapshot::ProjectSnapshot;
use std::error::Error;

#[async_trait::async_trait]
pub trait ProjectRepository {
    type Error;

    async fn get_project(&self, project_id: &str) -> Result<Option<Project>, Self::Error>;
}

#[async_trait::async_trait]
pub trait ProjectEditingSessionRepository {
    type Error;

    async fn create_session(&self, session: ProjectEditingSession) -> Result<(), Self::Error>;
    async fn get_active_session(
        &self,
        project_id: &str,
    ) -> Result<Option<ProjectEditingSession>, Self::Error>;
    async fn update_session(&self, session: ProjectEditingSession) -> Result<(), Self::Error>;
}

#[async_trait::async_trait]
pub trait ProjectSnapshotRepository {
    type Error: Error + Send + Sync + 'static;

    async fn create_snapshot(&self, snapshot: ProjectSnapshot) -> Result<(), Self::Error>;
    async fn update_latest_snapshot(&self, snapshot: ProjectSnapshot) -> Result<(), Self::Error>;
    async fn get_latest_snapshot(
        &self,
        project_id: &str,
    ) -> Result<Option<ProjectSnapshot>, Self::Error>;
    async fn get_latest_snapshot_state(&self, project_id: &str) -> Result<Vec<u8>, Self::Error>;
    async fn update_snapshot(&self, snapshot: ProjectSnapshot) -> Result<(), Self::Error>;
}
