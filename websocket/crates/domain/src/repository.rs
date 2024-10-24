use crate::editing_session::ProjectEditingSession;
use crate::project_type::Project;
use crate::types::data::SnapshotData;
use crate::types::snapshot::ProjectSnapshot;
use std::error::Error;

#[async_trait::async_trait]
pub trait ProjectRepository {
    type Error: Error + Send + Sync + 'static;

    async fn get_project(&self, project_id: &str) -> Result<Option<Project>, Self::Error>;
}

#[async_trait::async_trait]
pub trait ProjectEditingSessionRepository {
    type Error: Error + Send + Sync + 'static;

    async fn create_session(&self, session: ProjectEditingSession) -> Result<String, Self::Error>;
    async fn get_active_session(
        &self,
        project_id: &str,
    ) -> Result<Option<ProjectEditingSession>, Self::Error>;
    async fn update_session(&self, session: ProjectEditingSession) -> Result<(), Self::Error>;
    async fn get_client_count(&self) -> Result<usize, Self::Error>;
}

#[async_trait::async_trait]
pub trait ProjectSnapshotRepository {
    type Error: Error + Send + Sync + 'static;

    async fn create_snapshot(&self, snapshot: ProjectSnapshot) -> Result<(), Self::Error>;
    async fn create_snapshot_state(&self, snapshot_data: SnapshotData) -> Result<(), Self::Error>;
    async fn get_latest_snapshot(
        &self,
        project_id: &str,
    ) -> Result<Option<ProjectSnapshot>, Self::Error>;
    async fn get_latest_snapshot_state(&self, project_id: &str) -> Result<Vec<u8>, Self::Error>;
    async fn update_latest_snapshot(&self, snapshot: ProjectSnapshot) -> Result<(), Self::Error>;
    async fn update_latest_snapshot_state(
        &self,
        project_id: &str,
        snapshot_data: SnapshotData,
    ) -> Result<(), Self::Error>;
    async fn delete_snapshot_state(&self, project_id: &str) -> Result<(), Self::Error>;
}

#[async_trait::async_trait]
pub trait RedisDataManager {
    type Error: Error + Send + Sync + 'static;

    async fn merge_updates(&self, skip_lock: bool) -> Result<(Vec<u8>, Vec<String>), Self::Error>;
    async fn get_current_state(&self) -> Result<Option<Vec<u8>>, Self::Error>;
    async fn push_update(
        &self,
        update: Vec<u8>,
        updated_by: Option<String>,
    ) -> Result<(), Self::Error>;
    async fn clear_data(&self) -> Result<(), Self::Error>;
}
