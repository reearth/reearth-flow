/// Traits defining repository interfaces for project data management
use crate::persistence::editing_session::ProjectEditingSession;
use crate::types::project::Project;
use crate::types::snapshot::ProjectSnapshot;
use std::error::Error;

/// Trait for basic project operations
#[async_trait::async_trait]
pub trait ProjectImpl {
    /// The error type returned by project operations
    type Error: Error + Send + Sync + 'static;

    /// Gets a project by ID
    async fn get_project(&self, project_id: &str) -> Result<Option<Project>, Self::Error>;
}

/// Trait for project editing session operations
#[async_trait::async_trait]
pub trait ProjectEditingSessionImpl {
    /// The error type returned by session operations
    type Error: Error + Send + Sync + 'static;

    /// Creates a new editing session
    async fn create_session(&self, session: ProjectEditingSession) -> Result<String, Self::Error>;
    /// Gets the active editing session for a project
    async fn get_active_session(
        &self,
        project_id: &str,
    ) -> Result<Option<ProjectEditingSession>, Self::Error>;
    /// Updates an existing editing session
    async fn update_session(&self, session: ProjectEditingSession) -> Result<(), Self::Error>;
    /// Deletes an editing session
    async fn delete_session(&self, project_id: &str) -> Result<(), Self::Error>;
}

/// Trait for project snapshot operations
#[async_trait::async_trait]
pub trait ProjectSnapshotImpl {
    /// The error type returned by snapshot operations
    type Error: Error + Send + Sync + 'static;

    /// Creates a new snapshot
    async fn create_snapshot(&self, snapshot: ProjectSnapshot) -> Result<(), Self::Error>;
    /// Gets the latest snapshot for a project
    async fn get_latest_snapshot(
        &self,
        project_id: &str,
    ) -> Result<Option<ProjectSnapshot>, Self::Error>;
    /// Lists all snapshot versions for a project
    async fn list_all_snapshots_versions(
        &self,
        project_id: &str,
    ) -> Result<Vec<String>, Self::Error>;
    /// Updates the latest snapshot
    async fn update_latest_snapshot(&self, snapshot: ProjectSnapshot) -> Result<(), Self::Error>;
    /// Deletes a snapshot
    async fn delete_snapshot(&self, project_id: &str) -> Result<(), Self::Error>;
}

/// Trait for Redis data management operations
#[async_trait::async_trait]
pub trait RedisDataManagerImpl {
    /// The error type returned by Redis operations
    type Error: Error + Send + Sync + 'static;

    /// Creates a new Redis session
    async fn create_session(&self, project_id: &str, session_id: &str) -> Result<(), Self::Error>;
    /// Merges all pending updates for a project
    async fn merge_updates(
        &self,
        project_id: &str,
        skip_lock: bool,
    ) -> Result<(Vec<u8>, Vec<String>), Self::Error>;
    /// Gets the current state of a project
    async fn get_current_state(
        &self,
        project_id: &str,
        session_id: Option<&str>,
    ) -> Result<Option<Vec<u8>>, Self::Error>;
    /// Pushes a new update to the project state
    async fn push_update(
        &self,
        project_id: &str,
        update_data: Vec<u8>,
        updated_by: Option<String>,
    ) -> Result<(), Self::Error>;
    /// Clears all data for a project
    async fn clear_data(
        &self,
        project_id: &str,
        session_id: Option<&str>,
    ) -> Result<(), Self::Error>;
    /// Gets the ID of the active session for a project
    async fn get_active_session_id(&self, project_id: &str) -> Result<Option<String>, Self::Error>;
}
