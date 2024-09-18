use std::error::Error;

use crate::project::{Project, ProjectEditingSession};
use crate::snapshot::ProjectSnapshot;

#[async_trait::async_trait]
pub trait ProjectRepository {
    async fn get_project(&self, project_id: &str) -> Result<Option<Project>, Box<dyn Error>>;
}

#[async_trait::async_trait]
pub trait ProjectEditingSessionRepository {
    async fn create_session(&self, session: ProjectEditingSession) -> Result<(), Box<dyn Error>>;
    async fn get_active_session(
        &self,
        project_id: &str,
    ) -> Result<Option<ProjectEditingSession>, Box<dyn Error>>;
    async fn update_session(&self, session: ProjectEditingSession) -> Result<(), Box<dyn Error>>;
}

#[async_trait::async_trait]
pub trait ProjectSnapshotRepository {
    async fn create_snapshot(&self, snapshot: ProjectSnapshot) -> Result<(), Box<dyn Error>>;
    async fn get_latest_snapshot(
        &self,
        project_id: &str,
    ) -> Result<Option<ProjectSnapshot>, Box<dyn Error>>;
    async fn get_latest_snapshot_state(&self, project_id: &str) -> Result<Vec<u8>, Box<dyn Error>>;
}
