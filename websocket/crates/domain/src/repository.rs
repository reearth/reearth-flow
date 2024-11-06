use crate::editing_session::ProjectEditingSession;
use crate::project::Project;
use crate::types::snapshot::ProjectSnapshot;
use std::error::Error;

#[async_trait::async_trait]
pub trait ProjectImpl {
    type Error: Error + Send + Sync + 'static;

    async fn get_project(&self, project_id: &str) -> Result<Option<Project>, Self::Error>;
}

#[async_trait::async_trait]
pub trait ProjectEditingSessionImpl {
    type Error: Error + Send + Sync + 'static;

    async fn create_session(&self, session: ProjectEditingSession) -> Result<String, Self::Error>;
    async fn get_active_session(
        &self,
        project_id: &str,
    ) -> Result<Option<ProjectEditingSession>, Self::Error>;
    async fn update_session(&self, session: ProjectEditingSession) -> Result<(), Self::Error>;
    async fn delete_session(&self, project_id: &str) -> Result<(), Self::Error>;
}

#[async_trait::async_trait]
pub trait ProjectSnapshotImpl {
    type Error: Error + Send + Sync + 'static;

    async fn create_snapshot(&self, snapshot: ProjectSnapshot) -> Result<(), Self::Error>;
    async fn get_latest_snapshot(
        &self,
        project_id: &str,
    ) -> Result<Option<ProjectSnapshot>, Self::Error>;
    async fn list_all_snapshots_versions(
        &self,
        project_id: &str,
    ) -> Result<Vec<String>, Self::Error>;
    async fn update_latest_snapshot(&self, snapshot: ProjectSnapshot) -> Result<(), Self::Error>;
    async fn delete_snapshot(&self, project_id: &str) -> Result<(), Self::Error>;
}

#[async_trait::async_trait]
pub trait RedisDataManagerImpl {
    type Error: Error + Send + Sync + 'static;

    async fn create_session(&self, project_id: &str, session_id: &str) -> Result<(), Self::Error>;
    async fn merge_updates(
        &self,
        project_id: &str,
        skip_lock: bool,
    ) -> Result<(Vec<u8>, Vec<String>), Self::Error>;
    async fn get_current_state(
        &self,
        project_id: &str,
        session_id: Option<&str>,
    ) -> Result<Option<Vec<u8>>, Self::Error>;
    async fn push_update(
        &self,
        project_id: &str,
        update_data: Vec<u8>,
        updated_by: Option<String>,
    ) -> Result<(), Self::Error>;
    async fn clear_data(
        &self,
        project_id: &str,
        session_id: Option<&str>,
    ) -> Result<(), Self::Error>;
    async fn get_active_session_id(&self, project_id: &str) -> Result<Option<String>, Self::Error>;
}
