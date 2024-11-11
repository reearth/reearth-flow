use std::sync::Arc;

use crate::generate_id;
use crate::persistence::repository::{
    ProjectEditingSessionImpl, ProjectSnapshotImpl, RedisDataManagerImpl,
};
use crate::types::snapshot::ObjectTenant;
use crate::types::snapshot::{Metadata, ObjectDelete, ProjectSnapshot, SnapshotInfo};
use crate::types::user::User;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::Mutex;
use tracing::debug;

use super::project_repository::ProjectRepositoryError;
use super::redis::errors::FlowProjectRedisDataManagerError;

/// A guard that holds a mutex lock for the duration of its lifetime
struct SessionLockGuard<'a> {
    _lock: tokio::sync::MutexGuard<'a, ()>,
}

/// Represents an editing session for a project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectEditingSession {
    /// The ID of the project being edited
    pub project_id: String,
    /// The ID of the current editing session, if one exists
    pub session_id: Option<String>,
    /// A mutex lock to ensure thread-safe access to session data
    #[serde(skip)]
    session_lock: Arc<Mutex<()>>,
}

/// Errors that can occur during project editing session operations
#[derive(Error, Debug)]
pub enum ProjectEditingSessionError {
    /// Error when trying to perform operations without an active session
    #[error("Session not setup")]
    SessionNotSetup,
    /// Error when a snapshot cannot be found for a given session ID
    #[error("Snapshot not found for session ID: {0}")]
    SnapshotNotFound(String),
    /// Error when a snapshot's project ID doesn't match the current project
    #[error("Snapshot project ID does not match current project")]
    SnapshotProjectIdMismatch,
    /// Error from snapshot repository operations
    #[error(transparent)]
    Snapshot(#[from] ProjectRepositoryError),
    /// Error from Redis operations
    #[error(transparent)]
    Redis(#[from] FlowProjectRedisDataManagerError),
}

/// Default implementation for ProjectEditingSession
impl Default for ProjectEditingSession {
    fn default() -> Self {
        Self {
            project_id: "".to_string(),
            session_id: None,
            session_lock: Arc::new(Mutex::new(())),
        }
    }
}

impl ProjectEditingSession {
    /// Acquires a lock on the session
    async fn acquire_lock(&self) -> SessionLockGuard<'_> {
        SessionLockGuard {
            _lock: self.session_lock.lock().await,
        }
    }

    /// Creates a new ProjectEditingSession with the given project ID
    pub fn new(project_id: String) -> Self {
        Self {
            project_id,
            session_id: None,
            session_lock: Arc::new(Mutex::new(())),
        }
    }

    /// Starts a new editing session or joins an existing one
    pub async fn start_or_join_session<S, E, R>(
        &mut self,
        snapshot_repo: &S,
        project_editing_session_repository: &E,
        redis_manager: &R,
        user: &User,
    ) -> Result<(), ProjectEditingSessionError>
    where
        E: ProjectEditingSessionImpl<Error = ProjectRepositoryError>,
        S: ProjectSnapshotImpl<Error = ProjectRepositoryError>,
        R: RedisDataManagerImpl<Error = FlowProjectRedisDataManagerError>,
    {
        if let Some(project_editing_session) = project_editing_session_repository
            .get_active_session(&self.project_id)
            .await?
        {
            self.session_id = project_editing_session.session_id.clone();
            debug!("Joined existing session for project: {}", self.project_id);
            return Ok(());
        }

        let project_editing_session = ProjectEditingSession::new(self.project_id.clone());

        project_editing_session_repository
            .create_session(project_editing_session)
            .await?;

        debug!("Created new session for project: {}", self.project_id);

        self.load_session(snapshot_repo, redis_manager, user).await
    }

    /// Loads the session data from a snapshot or creates a new snapshot if none exists
    async fn load_session<R, S>(
        &self,
        snapshot_repo: &S,
        redis_manager: &R,
        user: &User,
    ) -> Result<(), ProjectEditingSessionError>
    where
        R: RedisDataManagerImpl<Error = FlowProjectRedisDataManagerError>,
        S: ProjectSnapshotImpl<Error = ProjectRepositoryError>,
    {
        if let Some(snapshot) = snapshot_repo.get_latest_snapshot(&self.project_id).await? {
            debug!("Found existing snapshot for project: {}", self.project_id);
            redis_manager
                .push_update(&self.project_id, snapshot.data, Some(user.name.clone()))
                .await?;
        } else {
            debug!(
                "No existing snapshot found for project: {}",
                self.project_id
            );
            self.create_snapshot(user, snapshot_repo, Vec::new(), None)
                .await?;
            debug!("Created new snapshot for project: {}", self.project_id);
        }
        Ok(())
    }

    /// Merges all pending updates for the project
    pub async fn merge_updates<R>(
        &self,
        redis_data_manager: &R,
    ) -> Result<(Vec<u8>, Vec<String>), ProjectEditingSessionError>
    where
        R: RedisDataManagerImpl<Error = FlowProjectRedisDataManagerError>,
    {
        self.check_session_setup()?;
        let _guard = self.acquire_lock().await;
        redis_data_manager
            .merge_updates(&self.project_id, false)
            .await
            .map_err(Into::into)
    }

    /// Gets the current state of the project
    pub async fn get_state_update<R>(
        &self,
        redis_data_manager: &R,
    ) -> Result<Vec<u8>, ProjectEditingSessionError>
    where
        R: RedisDataManagerImpl<Error = FlowProjectRedisDataManagerError>,
    {
        self.check_session_setup()?;

        let current_state = redis_data_manager
            .get_current_state(&self.project_id, self.session_id.as_deref())
            .await?;

        match current_state {
            Some(state) => Ok(state),
            None => Ok(Vec::new()),
        }
    }

    /// Pushes a new update to the project state
    pub async fn push_update<R>(
        &self,
        update: Vec<u8>,
        updated_by: String,
        redis_data_manager: &R,
    ) -> Result<(), ProjectEditingSessionError>
    where
        R: RedisDataManagerImpl<Error = FlowProjectRedisDataManagerError>,
    {
        self.check_session_setup()?;
        let _guard = self.acquire_lock().await;
        redis_data_manager
            .push_update(&self.project_id, update, Some(updated_by))
            .await
            .map_err(Into::into)
    }

    /// Creates a new snapshot of the project state
    pub async fn create_snapshot<S>(
        &self,
        user: &User,
        snapshot_repo: &S,
        data: Vec<u8>,
        snapshot_name: Option<String>,
    ) -> Result<(), ProjectEditingSessionError>
    where
        S: ProjectSnapshotImpl<Error = ProjectRepositoryError>,
    {
        let _guard = self.acquire_lock().await;

        let now = Utc::now();
        let metadata = Metadata::new(
            generate_id!("snap"),
            self.project_id.clone(),
            self.session_id.clone(),
            snapshot_name,
            String::new(),
        );

        let snapshot_info = SnapshotInfo::new(
            user.name.clone(),
            vec![],
            ObjectTenant::new(user.id.clone(), user.tenant_id.clone()),
            ObjectDelete::new(false, None),
            Some(now),
            None,
        );

        let snapshot = ProjectSnapshot::new(metadata, snapshot_info, data);
        snapshot_repo.create_snapshot(snapshot).await?;
        Ok(())
    }

    /// Ends the current editing session
    pub async fn end_session<R, S>(
        &mut self,
        redis_data_manager: &R,
        snapshot_repo: &S,
        snapshot_name: String,
        save_changes: bool,
    ) -> Result<(), ProjectEditingSessionError>
    where
        R: RedisDataManagerImpl<Error = FlowProjectRedisDataManagerError>,
        S: ProjectSnapshotImpl<Error = ProjectRepositoryError>,
    {
        self.check_session_setup()?;

        {
            let _guard = self.acquire_lock().await;

            let (state, edits) = redis_data_manager
                .merge_updates(&self.project_id, true)
                .await?;

            debug!("state: {:?}", state);

            if save_changes {
                let snapshot = snapshot_repo.get_latest_snapshot(&self.project_id).await?;
                debug!("snapshot: {:?}", snapshot);

                if let Some(mut snapshot) = snapshot {
                    snapshot.data = state;
                    snapshot.info.changes_by = edits;
                    snapshot.metadata.name = Some(snapshot_name);
                    snapshot_repo.update_latest_snapshot(snapshot).await?;
                }
            }

            redis_data_manager
                .clear_data(&self.project_id, self.session_id.as_deref())
                .await?;
        }

        self.session_id = None;
        Ok(())
    }

    /// Gets the ID of the active editing session, if one exists
    pub async fn active_editing_session(
        &self,
    ) -> Result<Option<String>, ProjectEditingSessionError> {
        self.check_session_setup()?;
        Ok(self.session_id.clone())
    }

    /// Checks if a session is currently set up
    fn check_session_setup(&self) -> Result<(), ProjectEditingSessionError> {
        match &self.session_id {
            Some(_) => Ok(()),
            None => Err(ProjectEditingSessionError::SessionNotSetup),
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use async_trait::async_trait;
//     use mockall::{mock, predicate::eq};

//     // Mock RedisDataManagerImpl
//     mock! {
//         pub RedisDataManagerImpl {}

//         #[async_trait]
//         impl RedisDataManagerImpl for RedisDataManagerImpl {

//             type Error = ProjectEditingSessionError;
//             async fn push_update(
//                 &self,
//                 state: Vec<u8>,
//                 updated_by: Option<String>,
//             ) -> Result<(), ProjectEditingSessionError>;

//             async fn get_current_state(&self) -> Result<Option<Vec<u8>>, ProjectEditingSessionError>;

//             async fn merge_updates(&self, final_merge: bool) -> Result<(Vec<u8>, Vec<String>), ProjectEditingSessionError>;

//             async fn clear_data(&self) -> Result<(), ProjectEditingSessionError>;
//             async fn get_active_session_id(&self) -> Result<Option<String>, ProjectEditingSessionError>;
//             async fn set_active_session_id(&self, session_id: &str) -> Result<(), ProjectEditingSessionError>;
//         }
//     }

//     // Mock ProjectSnapshotImpl
//     mock! {
//         pub ProjectSnapshotImpl {}

//         #[async_trait]
//         impl ProjectSnapshotImpl for ProjectSnapshotImpl {
//             type Error = ProjectEditingSessionError;

//             async fn update_latest_snapshot(
//                 &self,
//                 snapshot: ProjectSnapshot,
//             ) -> Result<(), ProjectEditingSessionError>;

//             async fn create_snapshot_state(
//                 &self,
//                 snapshot_data: SnapshotData,
//             ) -> Result<(), ProjectEditingSessionError>;

//             async fn get_latest_snapshot_state(
//                 &self,
//                 project_id: &str,
//             ) -> Result<Vec<u8>, ProjectEditingSessionError>;

//             async fn create_snapshot(
//                 &self,
//                 snapshot: ProjectSnapshot,
//             ) -> Result<(), ProjectEditingSessionError>;

//             async fn get_latest_snapshot(
//                 &self,
//                 session_id: &str,
//             ) -> Result<Option<ProjectSnapshot>, ProjectEditingSessionError>;

//             async fn update_latest_snapshot_state(
//                 &self,
//                 project_id: &str,
//                 data: SnapshotData,
//             ) -> Result<(), ProjectEditingSessionError>;

//             async fn delete_snapshot_state(&self, project_id: &str) -> Result<(), ProjectEditingSessionError>;
//         }
//     }

//     #[tokio::test]
//     async fn test_start_or_join_session() {
//         let mut session = create_test_session();
//         let mut mock_snapshot_repo = MockProjectSnapshotRepository::new();
//         let mut mock_redis_manager = MockRedisDataManager::new();

//         mock_redis_manager
//             .expect_get_active_session_id()
//             .times(1)
//             .returning(|| Ok(None));

//         mock_redis_manager
//             .expect_set_active_session_id()
//             .times(1)
//             .returning(|_| Ok(()));

//         mock_snapshot_repo
//             .expect_get_latest_snapshot()
//             .times(1)
//             .returning(|_| Ok(Some(create_test_snapshot())));

//         mock_snapshot_repo
//             .expect_get_latest_snapshot_state()
//             .times(1)
//             .returning(|_| Ok(vec![1, 2, 3]));

//         mock_redis_manager
//             .expect_push_update()
//             .times(1)
//             .returning(|_, _| Ok(()));

//         let result = session
//             .start_or_join_session(&mock_snapshot_repo, &mock_redis_manager)
//             .await;

//         assert!(result.is_ok());
//         assert!(session.session_id.is_some());
//     }

//     fn create_test_session() -> ProjectEditingSession {
//         ProjectEditingSession::new(
//             "test_project".to_string(),
//             ObjectTenant::new(generate_id(14, "tenant"), "tenant".to_owned()),
//         )
//     }

//     fn create_test_snapshot() -> ProjectSnapshot {
//         ProjectSnapshot::new(
//             Metadata::new(
//                 generate_id(14, "snap"),
//                 "test_project".to_string(),
//                 Some("session_456".to_string()),
//                 "Test Snapshot".to_string(),
//                 "".to_string(),
//             ),
//             SnapshotInfo::new(
//                 Some("test_user".to_string()),
//                 vec![],
//                 ObjectTenant::new(generate_id(14, "tenant"), "tenant".to_owned()),
//                 ObjectDelete {
//                     deleted: false,
//                     delete_after: None,
//                 },
//                 Some(Utc::now()),
//                 None,
//             ),
//         )
//     }

//     #[tokio::test]
//     async fn test_get_diff_update() {
//         let mut session = ProjectEditingSession::new(
//             "test_project".to_string(),
//             ObjectTenant::new("tenant_id".to_string(), "tenant_name".to_string()),
//         );
//         session.session_id = Some("test_session".to_string());

//         // Test case 1: Current state matches input state
//         {
//             let mut mock_redis_manager = MockRedisDataManager::new();
//             mock_redis_manager
//                 .expect_get_current_state()
//                 .return_once(|| Ok(Some(vec![1, 2, 3])));

//             let result = session
//                 .get_diff_update(vec![1, 2, 3], &mock_redis_manager)
//                 .await;

//             assert!(result.is_ok());
//             let (diff, server_state) = result.unwrap();
//             assert_eq!(diff, Vec::<u8>::new());
//             assert_eq!(server_state, vec![1, 2, 3]);
//         }

//         // Test case 2: Current state differs from input state
//         {
//             let mut mock_redis_manager = MockRedisDataManager::new();
//             mock_redis_manager
//                 .expect_get_current_state()
//                 .return_once(|| Ok(Some(vec![4, 5, 6])));

//             let result = session
//                 .get_diff_update(vec![1, 2, 3], &mock_redis_manager)
//                 .await;

//             assert!(result.is_ok());
//             let (diff, server_state) = result.unwrap();
//             assert_ne!(diff, Vec::<u8>::new()); // The diff should not be empty
//             assert_eq!(server_state, vec![4, 5, 6]);
//         }

//         // Test case 3: No current state in Redis
//         {
//             let mut mock_redis_manager = MockRedisDataManager::new();
//             mock_redis_manager
//                 .expect_get_current_state()
//                 .return_once(|| Ok(None));

//             let result = session
//                 .get_diff_update(vec![1, 2, 3], &mock_redis_manager)
//                 .await;

//             assert!(result.is_ok());
//             let (diff, server_state) = result.unwrap();
//             assert_eq!(diff, vec![1, 2, 3]);
//             assert_eq!(server_state, Vec::<u8>::new());
//         }
//     }

//     #[tokio::test]
//     async fn test_merge_updates() {
//         let mut session = ProjectEditingSession::new(
//             "test_project".to_string(),
//             ObjectTenant::new("tenant_id".to_string(), "tenant_key".to_string()),
//         );
//         session.session_id = Some("test_session".to_string());

//         let mut mock_redis = MockRedisDataManager::new();
//         mock_redis
//             .expect_merge_updates()
//             .with(eq(false))
//             .times(1)
//             .returning(|_| Ok((vec![1, 2, 3], vec!["user1".to_string()])));

//         let result = session.merge_updates(&mock_redis).await;
//         assert!(result.is_ok());

//         if let Ok((state, _)) = result {
//             assert_eq!(state, vec![1, 2, 3]);
//         }
//     }
// }
