use super::utils::calculate_diff;
use std::sync::Arc;

use crate::repository::{ProjectSnapshotRepository, RedisDataManager};
use crate::types::data::SnapshotData;
use crate::types::snapshot::{Metadata, ObjectDelete, ObjectTenant, ProjectSnapshot, SnapshotInfo};
use crate::utils::generate_id;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectEditingSession {
    pub project_id: String,
    pub session_id: Option<String>,
    pub session_setup_complete: bool,
    pub tenant: ObjectTenant,
    #[serde(skip)]
    session_lock: Arc<Mutex<()>>,
}

#[derive(Error, Debug)]
pub enum ProjectEditingSessionError {
    #[error("Session not setup")]
    SessionNotSetup,
    #[error("Snapshot not found")]
    SnapshotNotFound,
    #[error("Snapshot project ID does not match current project")]
    SnapshotProjectIdMismatch,
    #[error("Snapshot error: {0}")]
    Snapshot(String),
    #[error("Redis error: {0}")]
    Redis(String),
    #[error("{0}")]
    Custom(String),
}

impl ProjectEditingSessionError {
    pub fn snapshot<E: std::fmt::Display>(err: E) -> Self {
        Self::Snapshot(err.to_string())
    }

    pub fn redis<E: std::fmt::Display>(err: E) -> Self {
        Self::Redis(err.to_string())
    }
}

impl Default for ProjectEditingSession {
    fn default() -> Self {
        Self {
            project_id: "".to_string(),
            session_id: None,
            session_setup_complete: false,
            session_lock: Arc::new(Mutex::new(())),
            tenant: ObjectTenant::new(generate_id(14, "tenant"), "tenant".to_owned()),
        }
    }
}

impl ProjectEditingSession {
    pub fn new(project_id: String, tenant: ObjectTenant) -> Self {
        Self {
            project_id,
            session_id: None,
            tenant,
            session_setup_complete: false,
            session_lock: Arc::new(Mutex::new(())),
        }
    }

    pub async fn start_or_join_session<S, R>(
        &mut self,
        snapshot_repo: &S,
        redis_data_manager: &R,
    ) -> Result<String, ProjectEditingSessionError>
    where
        S: ProjectSnapshotRepository + ?Sized,
        R: RedisDataManager,
    {
        let session_id = generate_id(14, "editor-session");
        self.session_id = Some(session_id.clone());
        if !self.session_setup_complete {
            let latest_snapshot_state = snapshot_repo
                .get_latest_snapshot_state(&self.project_id)
                .await
                .map_err(ProjectEditingSessionError::snapshot)?;
            redis_data_manager
                .push_update(latest_snapshot_state, None)
                .await
                .map_err(ProjectEditingSessionError::redis)?;
        }
        self.session_setup_complete = true;
        Ok(session_id)
    }

    pub async fn get_diff_update<R>(
        &self,
        state_vector: Vec<u8>,
        redis_data_manager: &R,
    ) -> Result<(Vec<u8>, Vec<u8>), ProjectEditingSessionError>
    where
        R: RedisDataManager,
    {
        if !self.session_setup_complete {
            return Err(ProjectEditingSessionError::SessionNotSetup);
        }

        let current_state = redis_data_manager
            .get_current_state()
            .await
            .map_err(ProjectEditingSessionError::redis)?;

        match current_state {
            Some(current_state) => {
                if current_state == state_vector {
                    Ok((Vec::new(), current_state))
                } else {
                    let (diff, server_state) = calculate_diff(&state_vector, &current_state);
                    Ok((diff, server_state))
                }
            }
            None => Ok((state_vector.clone(), Vec::new())),
        }
    }

    pub async fn merge_updates<R>(
        &self,
        redis_data_manager: &R,
    ) -> Result<(Vec<u8>, Vec<String>), ProjectEditingSessionError>
    where
        R: RedisDataManager,
    {
        self.check_session_setup()?;

        let _lock = self.session_lock.lock().await;
        redis_data_manager
            .merge_updates(false)
            .await
            .map_err(ProjectEditingSessionError::redis)
    }

    pub async fn get_state_update<R>(
        &self,
        redis_data_manager: &R,
    ) -> Result<Vec<u8>, ProjectEditingSessionError>
    where
        R: RedisDataManager,
    {
        if !self.session_setup_complete {
            return Err(ProjectEditingSessionError::SessionNotSetup);
        }
        let current_state = redis_data_manager
            .get_current_state()
            .await
            .map_err(ProjectEditingSessionError::redis)?;

        match current_state {
            Some(state) => Ok(state),
            None => Ok(Vec::new()),
        }
    }

    pub async fn push_update<R>(
        &self,
        update: Vec<u8>,
        updated_by: String,
        redis_data_manager: &R,
    ) -> Result<(), ProjectEditingSessionError>
    where
        R: RedisDataManager,
    {
        self.check_session_setup()?;

        let _lock = self.session_lock.lock().await;
        redis_data_manager
            .push_update(update, Some(updated_by))
            .await
            .map_err(ProjectEditingSessionError::redis)
    }

    pub async fn create_snapshot<R, S>(
        &self,
        snapshot_repo: &S,
        redis_data_manager: &R,
        data: SnapshotData,
    ) -> Result<(), ProjectEditingSessionError>
    where
        S: ProjectSnapshotRepository,
        R: RedisDataManager,
    {
        self.check_session_setup()?;

        let _lock = self.session_lock.lock().await;
        self.create_snapshot_internal(snapshot_repo, redis_data_manager, data)
            .await
    }

    async fn create_snapshot_internal<R, S>(
        &self,
        snapshot_repo: &S,
        redis_data_manager: &R,
        data: SnapshotData,
    ) -> Result<(), ProjectEditingSessionError>
    where
        S: ProjectSnapshotRepository,
        R: RedisDataManager,
    {
        self.merge_updates(redis_data_manager).await?;

        let now = Utc::now();

        let metadata = Metadata::new(
            generate_id(14, "snap"),
            self.project_id.clone(),
            self.session_id.clone(),
            data.name.unwrap_or_default(),
            String::new(),
        );

        let snapshot_info = SnapshotInfo::new(
            data.created_by,
            vec![],
            self.tenant.clone(),
            ObjectDelete {
                deleted: false,
                delete_after: None,
            },
            Some(now),
            None,
        );

        let snapshot = ProjectSnapshot::new(metadata, snapshot_info);

        snapshot_repo
            .create_snapshot(snapshot)
            .await
            .map_err(ProjectEditingSessionError::snapshot)?;
        Ok(())
    }

    pub async fn end_session<R, S>(
        &mut self,
        redis_data_manager: &R,
        snapshot_repo: &S,
    ) -> Result<(), ProjectEditingSessionError>
    where
        R: RedisDataManager,
        S: ProjectSnapshotRepository,
    {
        if !self.session_setup_complete {
            return Err(ProjectEditingSessionError::SessionNotSetup);
        }

        let _lock = self.session_lock.lock().await;

        let (state, _) = redis_data_manager
            .merge_updates(true)
            .await
            .map_err(ProjectEditingSessionError::redis)?;

        let snapshot_data = SnapshotData::new(self.project_id.clone(), state, None, None);

        snapshot_repo
            .update_latest_snapshot_state(&snapshot_data.project_id, snapshot_data.clone())
            .await
            .map_err(ProjectEditingSessionError::snapshot)?;

        redis_data_manager
            .clear_data()
            .await
            .map_err(ProjectEditingSessionError::redis)?;

        self.session_id = None;
        self.session_setup_complete = false;

        Ok(())
    }

    pub async fn load_session<S>(
        &mut self,
        snapshot_repo: &S,
        session_id: &str,
    ) -> Result<(), ProjectEditingSessionError>
    where
        S: ProjectSnapshotRepository,
    {
        let snapshot = snapshot_repo
            .get_latest_snapshot(session_id)
            .await
            .map_err(ProjectEditingSessionError::snapshot)?;
        if let Some(snapshot) = snapshot {
            if snapshot.metadata.project_id != self.project_id {
                return Err(ProjectEditingSessionError::SnapshotProjectIdMismatch);
            }
            self.project_id = snapshot.metadata.project_id;
            self.session_id = Some(session_id.to_string());
            self.session_setup_complete = true;
        } else {
            return Err(ProjectEditingSessionError::SnapshotNotFound);
        }
        Ok(())
    }

    pub async fn active_editing_session(
        &self,
    ) -> Result<Option<String>, ProjectEditingSessionError> {
        if !self.session_setup_complete {
            Err(ProjectEditingSessionError::SessionNotSetup)
        } else {
            Ok(self.session_id.clone())
        }
    }

    // Helper method to check if the session is set up
    fn check_session_setup(&self) -> Result<(), ProjectEditingSessionError> {
        if !self.session_setup_complete {
            Err(ProjectEditingSessionError::SessionNotSetup)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use mockall::mock;

    // Mock RedisDataManager
    mock! {
        pub RedisDataManager {}

        #[async_trait]
        impl RedisDataManager for RedisDataManager {

            type Error = ProjectEditingSessionError;
            async fn push_update(
                &self,
                state: Vec<u8>,
                updated_by: Option<String>,
            ) -> Result<(), ProjectEditingSessionError>;

            async fn get_current_state(&self) -> Result<Option<Vec<u8>>, ProjectEditingSessionError>;

            async fn merge_updates(&self, final_merge: bool) -> Result<(Vec<u8>, Vec<String>), ProjectEditingSessionError>;

            async fn clear_data(&self) -> Result<(), ProjectEditingSessionError>;
        }
    }

    // Mock ProjectSnapshotRepository
    mock! {
        pub ProjectSnapshotRepository {}

        #[async_trait]
        impl ProjectSnapshotRepository for ProjectSnapshotRepository {
            type Error = ProjectEditingSessionError;

            async fn update_latest_snapshot(
                &self,
                snapshot: ProjectSnapshot,
            ) -> Result<(), ProjectEditingSessionError>;

            async fn create_snapshot_state(
                &self,
                snapshot_data: SnapshotData,
            ) -> Result<(), ProjectEditingSessionError>;

            async fn get_latest_snapshot_state(
                &self,
                project_id: &str,
            ) -> Result<Vec<u8>, ProjectEditingSessionError>;

            async fn create_snapshot(
                &self,
                snapshot: ProjectSnapshot,
            ) -> Result<(), ProjectEditingSessionError>;

            async fn get_latest_snapshot(
                &self,
                session_id: &str,
            ) -> Result<Option<ProjectSnapshot>, ProjectEditingSessionError>;

            async fn update_latest_snapshot_state(
                &self,
                project_id: &str,
                data: SnapshotData,
            ) -> Result<(), ProjectEditingSessionError>;

            async fn delete_snapshot_state(&self, project_id: &str) -> Result<(), ProjectEditingSessionError>;
        }
    }

    #[tokio::test]
    async fn test_start_or_join_session() {
        let mut session = ProjectEditingSession::new(
            "test_project".to_string(),
            ObjectTenant::new("tenant_id".to_string(), "tenant_name".to_string()),
        );

        let mut mock_snapshot_repo = MockProjectSnapshotRepository::new();
        let mut mock_redis_manager = MockRedisDataManager::new();

        // Mock behavior for the latest snapshot state and Redis push_update
        mock_snapshot_repo
            .expect_get_latest_snapshot_state()
            .returning(|_| Ok(vec![1, 2, 3]));

        mock_redis_manager
            .expect_push_update()
            .returning(|_, _| Ok(()));

        // Call the method
        let session_id = session
            .start_or_join_session(&mock_snapshot_repo, &mock_redis_manager)
            .await;

        // Assert success and that session ID is set
        assert!(session_id.is_ok());
        assert!(session.session_id.is_some());
        assert!(session.session_setup_complete);
    }

    #[tokio::test]
    async fn test_get_diff_update() {
        let mut session = ProjectEditingSession::new(
            "test_project".to_string(),
            ObjectTenant::new("tenant_id".to_string(), "tenant_name".to_string()),
        );
        session.session_setup_complete = true;

        // Test case 1: Current state matches input state
        {
            let mut mock_redis_manager = MockRedisDataManager::new();
            mock_redis_manager
                .expect_get_current_state()
                .return_once(|| Ok(Some(vec![1, 2, 3])));

            let result = session
                .get_diff_update(vec![1, 2, 3], &mock_redis_manager)
                .await;

            assert!(result.is_ok());
            let (diff, server_state) = result.unwrap();
            assert_eq!(diff, Vec::<u8>::new());
            assert_eq!(server_state, vec![1, 2, 3]);
        }

        // Test case 2: Current state differs from input state
        {
            let mut mock_redis_manager = MockRedisDataManager::new();
            mock_redis_manager
                .expect_get_current_state()
                .return_once(|| Ok(Some(vec![4, 5, 6])));

            let result = session
                .get_diff_update(vec![1, 2, 3], &mock_redis_manager)
                .await;

            assert!(result.is_ok());
            let (diff, server_state) = result.unwrap();
            assert_ne!(diff, Vec::<u8>::new()); // The diff should not be empty
            assert_eq!(server_state, vec![4, 5, 6]);
        }

        // Test case 3: No current state in Redis
        {
            let mut mock_redis_manager = MockRedisDataManager::new();
            mock_redis_manager
                .expect_get_current_state()
                .return_once(|| Ok(None));

            let result = session
                .get_diff_update(vec![1, 2, 3], &mock_redis_manager)
                .await;

            assert!(result.is_ok());
            let (diff, server_state) = result.unwrap();
            assert_eq!(diff, vec![1, 2, 3]);
            assert_eq!(server_state, Vec::<u8>::new());
        }
    }

    #[tokio::test]
    async fn test_merge_updates() {
        let mut session = ProjectEditingSession::new(
            "test_project".to_string(),
            ObjectTenant::new("tenant_id".to_string(), "tenant_name".to_string()),
        );
        session.session_setup_complete = true; // Set this to true

        let mut mock_redis_manager = MockRedisDataManager::new();

        mock_redis_manager
            .expect_merge_updates()
            .returning(|_| Ok((vec![1, 2, 3], vec!["test_update".to_string()])));

        let result = session.merge_updates(&mock_redis_manager).await;

        assert!(result.is_ok());
        let (state, updates) = result.unwrap();
        assert_eq!(state, vec![1, 2, 3]);
        assert_eq!(updates, vec!["test_update"]);
    }
}
