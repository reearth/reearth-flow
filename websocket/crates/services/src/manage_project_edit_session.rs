use chrono::Utc;
use flow_websocket_domain::{
    editing_session::ProjectEditingSession,
    repository::{
        ProjectEditingSessionImpl, ProjectImpl, ProjectSnapshotImpl, RedisDataManagerImpl,
    },
    user::User,
};
use flow_websocket_infra::persistence::{
    project_repository::ProjectRepositoryError, redis::errors::FlowProjectRedisDataManagerError,
};
use mockall::automock;
use std::time::Duration;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{mpsc, Mutex};
use tokio::time::sleep;
use tracing::debug;

use crate::project::ProjectService;
use crate::{types::ManageProjectEditSessionTaskData, ProjectServiceError};

const MAX_EMPTY_SESSION_DURATION: Duration = Duration::from_secs(10);
//const MAX_SNAPSHOT_DELTA: Duration = Duration::from_secs(5 * 60);
const JOB_COMPLETION_DELAY: Duration = Duration::from_secs(5);

#[derive(Debug, Clone)]
pub struct ManageEditSessionService<R, S, M>
where
    R: ProjectEditingSessionImpl<Error = ProjectRepositoryError> + Send + Sync + Clone + 'static,
    S: ProjectSnapshotImpl<Error = ProjectRepositoryError> + Send + Sync + Clone + 'static,
    M: RedisDataManagerImpl<Error = FlowProjectRedisDataManagerError>
        + Send
        + Sync
        + Clone
        + 'static,
{
    pub project_service: ProjectService<R, S, M>,
    tasks: Arc<Mutex<HashMap<String, ManageProjectEditSessionTaskData>>>,
}

#[derive(Debug)]
pub enum SessionCommand {
    Start {
        project_id: String,
        user: User,
    },
    End {
        project_id: String,
        user: User,
    },
    Complete {
        project_id: String,
        user: User,
    },
    CheckStatus {
        project_id: String,
    },
    AddTask {
        task_data: ManageProjectEditSessionTaskData,
    },
    RemoveTask {
        project_id: String,
    },
    ListAllSnapshotsVersions {
        project_id: String,
    },
}

#[automock]
impl<R, S, M> ManageEditSessionService<R, S, M>
where
    R: ProjectEditingSessionImpl<Error = ProjectRepositoryError>
        + ProjectImpl<Error = ProjectRepositoryError>
        + Send
        + Sync
        + Clone
        + 'static,
    S: ProjectSnapshotImpl<Error = ProjectRepositoryError> + Send + Sync + Clone + 'static,
    M: RedisDataManagerImpl<Error = FlowProjectRedisDataManagerError>
        + Send
        + Sync
        + Clone
        + 'static,
{
    pub fn new(
        session_repository: Arc<R>,
        snapshot_repository: Arc<S>,
        redis_data_manager: Arc<M>,
    ) -> Self {
        Self {
            project_service: ProjectService::new(
                session_repository,
                snapshot_repository,
                redis_data_manager,
            ),
            tasks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn process(
        &self,
        mut command_rx: mpsc::Receiver<SessionCommand>,
    ) -> Result<(), ProjectServiceError> {
        loop {
            tokio::select! {
                Some(command) = command_rx.recv() => {
                    match command {
                        SessionCommand::Start { project_id, user } => {
                            let session = self.project_service
                                .get_or_create_editing_session(&project_id, user.clone())
                                .await?;

                            if session.session_id.is_some() {
                                debug!("Session exists/created for project: {}", project_id);
                                if let Some(task_data) = self.get_task_data(&project_id).await {
                                    let mut count = task_data.client_count.write().await;
                                    *count = Some(count.unwrap_or(0) + 1);
                                    debug!("Client count increased to: {:?}", *count);
                                }
                            }
                        },
                        SessionCommand::End { project_id, user } => {
                            if let Some(task_data) = self.get_task_data(&project_id).await {
                                {
                                    let mut count = task_data.client_count.write().await;
                                    if let Some(current_count) = *count {
                                        *count = Some(current_count.saturating_sub(1));
                                        debug!("Client count decreased to: {:?}", *count);
                                        if *count == Some(0) {
                                            let mut disconnected_at = task_data.clients_disconnected_at.write().await;
                                            *disconnected_at = Some(Utc::now());
                                            debug!("All clients disconnected at: {:?}", *disconnected_at);
                                        }
                                    }
                                }

                                if let Some(mut session) = self.get_latest_session(&project_id).await? {
                                    if let Ok(()) = self.end_editing_session_if_conditions_met(&mut session, &task_data).await {
                                        debug!("Session ended by user: {} for project: {}", user.name, project_id);
                                        break;
                                    }
                                }
                            }
                        },

                        SessionCommand::ListAllSnapshotsVersions { project_id } => {
                            let versions = self.project_service.list_all_snapshots_versions(&project_id).await?;
                            debug!("List of all snapshots versions for project: {}", project_id);
                            debug!("{:?}", versions);
                        },

                        SessionCommand::Complete { project_id, user } => {
                            if let Some(mut session) = self.get_latest_session(&project_id).await? {
                                if let Ok(()) = self.complete_job_if_met_requirements(&mut session).await {
                                    debug!("Job completed by user: {} for project: {}", user.name, project_id);
                                    break;
                                }
                            }
                        },

                        SessionCommand::CheckStatus { project_id } => {
                            debug!("Checking session status for project: {}", project_id);
                        },
                        SessionCommand::AddTask { task_data } => {
                            let mut tasks = self.tasks.lock().await;
                            tasks.insert(task_data.project_id.clone(), task_data.clone());
                            debug!("Added task for project: {}", task_data.project_id);
                        },
                        SessionCommand::RemoveTask { project_id } => {
                            let mut tasks = self.tasks.lock().await;
                            tasks.remove(&project_id);
                            debug!("Removed task for project: {}", project_id);
                        }
                    }
                },
                _ = tokio::time::sleep(Duration::from_secs(1)) => {
                    let tasks = self.tasks.lock().await;
                    for (project_id, data) in tasks.iter() {
                        if let Some(mut session) = self.get_latest_session(project_id).await? {
                            if let Ok(()) = self.end_editing_session_if_conditions_met(&mut session, data).await {
                                debug!("Session ended by condition check for project: {}", project_id);
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn get_task_data(&self, project_id: &str) -> Option<ManageProjectEditSessionTaskData> {
        let tasks = self.tasks.lock().await;
        tasks.get(project_id).cloned()
    }

    pub async fn get_latest_session(
        &self,
        project_id: &str,
    ) -> Result<Option<ProjectEditingSession>, ProjectServiceError> {
        let ret = self
            .project_service
            .session_repository
            .get_active_session(project_id)
            .await?;
        Ok(ret)
    }

    pub async fn end_editing_session_if_conditions_met(
        &self,
        session: &mut ProjectEditingSession,
        data: &ManageProjectEditSessionTaskData,
    ) -> Result<(), ProjectServiceError> {
        if let Some(client_count) = *data.client_count.read().await {
            if client_count == 0 {
                if let Some(clients_disconnected_at) = *data.clients_disconnected_at.read().await {
                    let current_time = Utc::now();
                    let clients_disconnection_elapsed_time = current_time - clients_disconnected_at;

                    if clients_disconnection_elapsed_time
                        .to_std()
                        .map_err(ProjectServiceError::ChronoDurationConversionError)?
                        > MAX_EMPTY_SESSION_DURATION
                    {
                        self.project_service
                            .end_session("system".to_string(), session.clone())
                            .await?;
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn complete_job_if_met_requirements(
        &self,
        session: &mut ProjectEditingSession,
    ) -> Result<(), ProjectServiceError> {
        self.project_service
            .end_session("system".to_string(), session.clone())
            .await?;

        sleep(JOB_COMPLETION_DELAY).await;

        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use chrono::{Duration, Utc};
//     use mockall::predicate::{self, *};

//     mockall::mock! {
//         ProjectEditingSessionImpl {}
//         #[async_trait::async_trait]
//         impl ProjectEditingSessionImpl for ProjectEditingSessionImpl {
//             type Error = ProjectRepositoryError;

//             async fn create_session(&self, session: ProjectEditingSession) -> Result<String, ProjectRepositoryError >;
//             async fn get_active_session(&self, project_id: &str) -> Result<Option<ProjectEditingSession>, ProjectRepositoryError>;
//             async fn update_session(&self, session: ProjectEditingSession) -> Result<(), ProjectRepositoryError>;
//             async fn get_client_count(&self) -> Result<usize, ProjectRepositoryError>;
//         }
//     }

//     mockall::mock! {
//     ProjectSnapshotImpl {}
//     #[async_trait::async_trait]
//     impl ProjectSnapshotImpl for ProjectSnapshotImpl {
//         type Error = ProjectRepositoryError;

//         async fn create_snapshot(&self, snapshot: ProjectSnapshot) -> Result<(), ProjectRepositoryError>;
//         async fn get_latest_snapshot(&self, project_id: &str) -> Result<Option<ProjectSnapshot>, ProjectRepositoryError>;
//         async fn get_latest_snapshot_state(&self, project_id: &str) -> Result<Vec<u8>, ProjectRepositoryError>;
//         async fn update_latest_snapshot(&self, snapshot: ProjectSnapshot) -> Result<(), ProjectRepositoryError>;
//         async fn update_latest_snapshot_state(&self, project_id: &str, snapshot_data: SnapshotData) -> Result<(), ProjectRepositoryError>;
//         async fn delete_snapshot_state(&self, project_id: &str) -> Result<(), ProjectRepositoryError>;
//         async fn create_snapshot_state(&self, snapshot_data: SnapshotData) -> Result<(), ProjectRepositoryError>;

//     }
//     }

//     mockall::mock! {
//         RedisDataManagerImpl {}
//         #[async_trait::async_trait]
//         impl RedisDataManagerImpl for RedisDataManagerImpl {
//             type Error = FlowProjectRedisDataManagerError;

//             async fn merge_updates(&self, skip_lock: bool) -> Result<(Vec<u8>, Vec<String>), FlowProjectRedisDataManagerError>;
//             async fn get_current_state(&self) -> Result<Option<Vec<u8>>, FlowProjectRedisDataManagerError>;
//             async fn push_update(&self, update: Vec<u8>, updated_by: Option<String>) -> Result<(), FlowProjectRedisDataManagerError>;
//             async fn clear_data(&self) -> Result<(), FlowProjectRedisDataManagerError>;
//             async fn get_active_session_id(&self) -> Result<Option<String>, FlowProjectRedisDataManagerError>;
//             async fn set_active_session_id(&self, session_id: &str) -> Result<(), FlowProjectRedisDataManagerError>;
//         }
//     }

//     #[tokio::test]
//     async fn test_process_with_active_session() {
//         let (mut service, mut mocks) = setup_service();
//         let task_data = create_task_data();
//         let session = create_session("project_123");

//         // Remove get_active_session expectation since it's not called anymore
//         mocks
//             .redis_manager
//             .expect_get_active_session_id()
//             .times(1)
//             .returning(|| Ok(Some("session_456".to_string())));

//         mocks
//             .redis_manager
//             .expect_merge_updates()
//             .with(eq(false))
//             .times(1)
//             .returning(|_| Ok((vec![], vec![])));

//         mocks
//             .session_repo
//             .expect_get_client_count()
//             .times(1)
//             .returning(|| Ok(1));

//         mocks
//             .session_repo
//             .expect_update_session()
//             .times(1)
//             .returning(|_| Ok(()));

//         service.session_repository = Arc::new(mocks.session_repo);
//         service.redis_data_manager = Arc::new(mocks.redis_manager);
//         service.snapshot_repository = Arc::new(mocks.snapshot_repo);

//         let result = service.process(task_data, None).await;
//         assert!(result.is_ok());
//     }

//     #[tokio::test]
//     async fn test_process_with_no_active_session() {
//         let (mut service, mut mocks) = setup_service();
//         let task_data = create_task_data();

//         mocks
//             .redis_manager
//             .expect_get_active_session_id()
//             .times(1)
//             .returning(|| Ok(None));

//         mocks
//             .redis_manager
//             .expect_set_active_session_id()
//             .times(1)
//             .returning(|_| Ok(()));

//         mocks
//             .snapshot_repo
//             .expect_get_latest_snapshot()
//             .times(1)
//             .returning(|_| Ok(Some(create_project_snapshot())));

//         mocks
//             .snapshot_repo
//             .expect_get_latest_snapshot_state()
//             .times(1)
//             .returning(|_| Ok(vec![1, 2, 3]));

//         mocks
//             .redis_manager
//             .expect_push_update()
//             .times(1)
//             .returning(|_, _| Ok(()));

//         mocks
//             .redis_manager
//             .expect_merge_updates()
//             .times(1)
//             .returning(|_| Ok((vec![], vec![])));

//         mocks
//             .session_repo
//             .expect_get_client_count()
//             .times(1)
//             .returning(|| Ok(1));

//         mocks
//             .session_repo
//             .expect_update_session()
//             .times(1)
//             .returning(|_| Ok(()));

//         service.session_repository = Arc::new(mocks.session_repo);
//         service.redis_data_manager = Arc::new(mocks.redis_manager);
//         service.snapshot_repository = Arc::new(mocks.snapshot_repo);

//         let result = service.process(task_data, None).await;
//         assert!(result.is_ok());
//     }

//     #[tokio::test]
//     async fn test_merge_updates() {
//         let (mut service, mut mocks) = setup_service();
//         let mut task_data = create_task_data();
//         let mut session = create_session("project_123");

//         session.session_id = Some("session_456".to_string());

//         mocks
//             .redis_manager
//             .expect_merge_updates()
//             .with(eq(false))
//             .times(1)
//             .returning(|_| Ok((vec![1, 2, 3], vec!["user1".to_string()])));

//         service.redis_data_manager = Arc::new(mocks.redis_manager);

//         let result = service.merge_updates(&mut session, &mut task_data).await;

//         if let Err(ref e) = result {
//             println!("Error in merge_updates: {:?}", e);
//         }

//         assert!(result.is_ok());
//         assert!(task_data.last_merged_at.is_some());
//     }

//     #[tokio::test]
//     async fn test_create_snapshot_if_required() {
//         // Test case 1: First snapshot creation
//         {
//             let (mut service, mut mocks) = setup_service();
//             let mut task_data = create_task_data();
//             task_data.last_snapshot_at = Some(Utc::now() - Duration::minutes(6));
//             let mut session = create_session("project_123");
//             session.session_id = Some("session_456".to_string());

//             // Set up sequence for get_latest_snapshot calls
//             let _context = mocks
//                 .snapshot_repo
//                 .expect_get_latest_snapshot()
//                 .with(eq("project_123"))
//                 .times(1)
//                 .returning(|_| Ok(None));

//             mocks
//                 .redis_manager
//                 .expect_get_current_state()
//                 .times(1)
//                 .returning(|| Ok(Some(vec![1, 2, 3])));

//             mocks
//                 .snapshot_repo
//                 .expect_create_snapshot()
//                 .times(1)
//                 .returning(|snapshot: ProjectSnapshot| {
//                     assert_eq!(snapshot.metadata.project_id, "project_123");
//                     assert_eq!(
//                         snapshot.metadata.session_id,
//                         Some("session_456".to_string())
//                     );
//                     assert_eq!(snapshot.info.created_by, Some("test_user".to_string()));
//                     assert!(snapshot.info.changes_by.is_empty());
//                     Ok(())
//                 });

//             mocks
//                 .snapshot_repo
//                 .expect_create_snapshot_state()
//                 .times(1)
//                 .returning(|snapshot_data: SnapshotData| {
//                     assert_eq!(snapshot_data.project_id, "project_123");
//                     assert_eq!(snapshot_data.state, vec![1, 2, 3]);
//                     Ok(())
//                 });

//             service.redis_data_manager = Arc::new(mocks.redis_manager);
//             service.snapshot_repository = Arc::new(mocks.snapshot_repo);

//             let result = service
//                 .create_snapshot_if_required(
//                     &mut session,
//                     &mut task_data,
//                     Some("test_user".to_string()),
//                 )
//                 .await;
//             assert!(result.is_ok());
//             assert!(task_data.last_snapshot_at.unwrap() > Utc::now() - Duration::seconds(1));
//         }

//         // Test case 2: Snapshot update
//         {
//             let (mut service, mut mocks) = setup_service();
//             let mut task_data = create_task_data();
//             task_data.last_snapshot_at = Some(Utc::now() - Duration::minutes(6));
//             let mut session = create_session("project_123");
//             session.session_id = Some("session_456".to_string());

//             // Set up sequence for get_latest_snapshot calls
//             let _context = mocks
//                 .snapshot_repo
//                 .expect_get_latest_snapshot()
//                 .with(eq("project_123"))
//                 .times(1)
//                 .returning(|_| Ok(Some(create_project_snapshot())));

//             mocks
//                 .redis_manager
//                 .expect_get_current_state()
//                 .times(1)
//                 .returning(|| Ok(Some(vec![1, 2, 3])));

//             mocks
//                 .snapshot_repo
//                 .expect_update_latest_snapshot()
//                 .times(1)
//                 .returning(|snapshot: ProjectSnapshot| {
//                     assert!(snapshot.info.changes_by.contains(&"test_user".to_string()));
//                     Ok(())
//                 });

//             mocks
//                 .snapshot_repo
//                 .expect_update_latest_snapshot_state()
//                 .times(1)
//                 .returning(|project_id: &str, snapshot_data: SnapshotData| {
//                     assert_eq!(project_id, "project_123");
//                     assert_eq!(snapshot_data.state, vec![1, 2, 3]);
//                     Ok(())
//                 });

//             service.redis_data_manager = Arc::new(mocks.redis_manager);
//             service.snapshot_repository = Arc::new(mocks.snapshot_repo);

//             let result = service
//                 .create_snapshot_if_required(
//                     &mut session,
//                     &mut task_data,
//                     Some("test_user".to_string()),
//                 )
//                 .await;
//             assert!(result.is_ok());
//             assert!(task_data.last_snapshot_at.unwrap() > Utc::now() - Duration::seconds(1));
//         }
//     }

//     #[tokio::test]
//     async fn test_end_editing_session_if_conditions_met() {
//         let (mut service, mut mocks) = setup_service();
//         let mut task_data = create_task_data();
//         task_data.clients_disconnected_at = Some(Utc::now() - Duration::seconds(11));
//         let mut session = create_session("project_123");
//         session.session_id = Some("session_456".to_string());

//         mocks
//             .redis_manager
//             .expect_merge_updates()
//             .with(eq(true))
//             .times(1)
//             .returning(|_| Ok((vec![1, 2, 3], vec!["user1".to_string()])));

//         mocks
//             .snapshot_repo
//             .expect_update_latest_snapshot_state()
//             .times(1)
//             .returning(|_, _| Ok(()));

//         mocks
//             .redis_manager
//             .expect_clear_data()
//             .times(1)
//             .returning(|| Ok(()));

//         service.session_repository = Arc::new(mocks.session_repo);
//         service.redis_data_manager = Arc::new(mocks.redis_manager);
//         service.snapshot_repository = Arc::new(mocks.snapshot_repo);

//         let result = service
//             .end_editing_session_if_conditions_met(&mut session, &task_data, 0)
//             .await;
//         assert!(result.is_ok());
//         assert!(session.session_id.is_none());
//     }

//     #[tokio::test]
//     async fn test_complete_job_if_met_requirements() {
//         let (mut service, mut mocks) = setup_service(); // Add mut
//         let mut session = create_session("project_123");

//         mocks
//             .redis_manager
//             .expect_merge_updates()
//             .with(eq(false))
//             .times(1)
//             .returning(|_| Ok((vec![1, 2, 3], vec![])));

//         // Add expectation for active_editing_session
//         session.session_id = Some("session_123".to_string());

//         // Add expectation for get_latest_snapshot
//         mocks
//             .snapshot_repo
//             .expect_get_latest_snapshot()
//             .with(eq("project_123"))
//             .times(1)
//             .returning(|_| Ok(Some(create_project_snapshot())));

//         // Add expectations for update_latest_snapshot and update_latest_snapshot_state
//         mocks
//             .snapshot_repo
//             .expect_update_latest_snapshot()
//             .times(1)
//             .returning(|_| Ok(()));

//         mocks
//             .snapshot_repo
//             .expect_update_latest_snapshot_state()
//             .times(1)
//             .returning(|_, _| Ok(()));

//         service.redis_data_manager = Arc::new(mocks.redis_manager);
//         service.snapshot_repository = Arc::new(mocks.snapshot_repo);

//         let result = service.complete_job_if_met_requirements(&session).await;
//         assert!(result.is_ok());
//     }

//     #[tokio::test]
//     async fn test_process_with_snapshot_creation() {
//         let (mut service, mut mocks) = setup_service();
//         let mut task_data = create_task_data();
//         task_data.last_snapshot_at = Some(Utc::now() - Duration::minutes(6));

//         // Set up all expectations before service calls
//         mocks
//             .redis_manager
//             .expect_get_active_session_id()
//             .times(1)
//             .returning(|| Ok(Some("session_456".to_string())));

//         mocks
//             .redis_manager
//             .expect_merge_updates()
//             .with(eq(false))
//             .times(1)
//             .returning(|_| Ok((vec![1, 2, 3], vec![])));

//         mocks
//             .session_repo
//             .expect_get_client_count()
//             .times(1)
//             .returning(|| Ok(1));

//         mocks
//             .snapshot_repo
//             .expect_get_latest_snapshot()
//             .times(1)
//             .returning(|_| Ok(Some(create_project_snapshot())));

//         mocks
//             .redis_manager
//             .expect_get_current_state()
//             .times(1)
//             .returning(|| Ok(Some(vec![1, 2, 3])));

//         mocks
//             .snapshot_repo
//             .expect_update_latest_snapshot()
//             .times(1)
//             .returning(|_| Ok(()));

//         mocks
//             .snapshot_repo
//             .expect_update_latest_snapshot_state()
//             .times(1)
//             .returning(|_, _| Ok(()));

//         mocks
//             .session_repo
//             .expect_update_session()
//             .times(1)
//             .returning(|_| Ok(()));

//         service.session_repository = Arc::new(mocks.session_repo);
//         service.redis_data_manager = Arc::new(mocks.redis_manager);
//         service.snapshot_repository = Arc::new(mocks.snapshot_repo);

//         let result = service
//             .process(task_data, Some("test_user".to_string()))
//             .await;
//         assert!(result.is_ok());
//     }

//     #[tokio::test]
//     async fn test_process_with_session_end() {
//         let (mut service, mut mocks) = setup_service();
//         let mut task_data = create_task_data();
//         task_data.clients_disconnected_at = Some(Utc::now() - Duration::seconds(11));

//         mocks
//             .redis_manager
//             .expect_get_active_session_id()
//             .times(1)
//             .returning(|| Ok(Some("session_456".to_string())));

//         mocks
//             .redis_manager
//             .expect_merge_updates()
//             .with(eq(false))
//             .times(1)
//             .returning(|_| Ok((vec![], vec![])));

//         mocks
//             .redis_manager
//             .expect_merge_updates()
//             .with(eq(true))
//             .times(1)
//             .returning(|_| Ok((vec![], vec![])));

//         mocks
//             .session_repo
//             .expect_get_client_count()
//             .times(1)
//             .returning(|| Ok(0));

//         // Add this expectation for updating snapshot state
//         mocks
//             .snapshot_repo
//             .expect_update_latest_snapshot_state()
//             .times(1)
//             .returning(|_, _| Ok(()));

//         mocks
//             .redis_manager
//             .expect_clear_data()
//             .times(1)
//             .returning(|| Ok(()));

//         service.session_repository = Arc::new(mocks.session_repo);
//         service.redis_data_manager = Arc::new(mocks.redis_manager);
//         service.snapshot_repository = Arc::new(mocks.snapshot_repo);

//         let result = service.process(task_data, None).await;
//         assert!(result.is_ok());
//     }

//     #[tokio::test]
//     async fn test_process_with_error() {
//         let (mut service, mut mocks) = setup_service();
//         let task_data = create_task_data();

//         mocks
//             .redis_manager
//             .expect_get_active_session_id()
//             .times(1)
//             .returning(|| {
//                 Err(FlowProjectRedisDataManagerError::Unknown(
//                     "Test error".to_string(),
//                 ))
//             });

//         service.redis_data_manager = Arc::new(mocks.redis_manager);

//         let result = service.process(task_data, None).await;
//         assert!(result.is_err());
//     }

//     // Helper functions

//     fn setup_service() -> (
//         ManageEditSessionService<
//             MockProjectEditingSessionRepository,
//             MockProjectSnapshotRepository,
//             MockRedisDataManager,
//         >,
//         MockRepositories,
//     ) {
//         let mock_session_repo = MockProjectEditingSessionRepository::new();
//         let mock_snapshot_repo = MockProjectSnapshotRepository::new();
//         let mock_redis_manager = MockRedisDataManager::new();

//         let service = ManageEditSessionService::new(
//             Arc::new(MockProjectEditingSessionRepository::new()),
//             Arc::new(MockProjectSnapshotRepository::new()),
//             Arc::new(MockRedisDataManager::new()),
//         );

//         let mocks = MockRepositories {
//             session_repo: mock_session_repo,
//             snapshot_repo: mock_snapshot_repo,
//             redis_manager: mock_redis_manager,
//         };

//         (service, mocks)
//     }

//     fn create_task_data() -> ManageProjectEditSessionTaskData {
//         ManageProjectEditSessionTaskData {
//             project_id: "project_123".to_string(),
//             clients_count: Some(1),
//             clients_disconnected_at: None,
//             last_merged_at: None,
//             last_snapshot_at: Some(Utc::now() - Duration::minutes(4)),
//         }
//     }

//     fn create_session(project_id: &str) -> ProjectEditingSession {
//         ProjectEditingSession::new(
//             project_id.to_string(),
//             ObjectTenant::new(generate_id(14, "tenant"), "tenant".to_owned()),
//         )
//     }

//     fn create_project_snapshot() -> ProjectSnapshot {
//         ProjectSnapshot::new(
//             Metadata::new(
//                 generate_id(14, "snap"),
//                 "project_123".to_string(),
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

//     struct MockRepositories {
//         session_repo: MockProjectEditingSessionRepository,
//         snapshot_repo: MockProjectSnapshotRepository,
//         redis_manager: MockRedisDataManager,
//     }
// }
