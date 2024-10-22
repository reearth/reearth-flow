use chrono::{DateTime, Utc};
use flow_websocket_domain::{
    generate_id,
    project::ProjectEditingSession,
    repository::{ProjectEditingSessionRepository, ProjectSnapshotRepository, RedisDataManager},
    snapshot::{Metadata, ObjectDelete, ObjectTenant, SnapshotInfo},
    types::{data::SnapshotData, snapshot::ProjectSnapshot},
};
use flow_websocket_infra::persistence::{
    project_repository::ProjectRepositoryError,
    redis::flow_project_redis_data_manager::FlowProjectRedisDataManagerError,
};
use mockall::automock;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::debug;

use crate::{types::ManageProjectEditSessionTaskData, ProjectServiceError};

const MAX_EMPTY_SESSION_DURATION: Duration = Duration::from_secs(10);
const MAX_SNAPSHOT_DELTA: Duration = Duration::from_secs(5 * 60);
const JOB_COMPLETION_DELAY: Duration = Duration::from_secs(5);

pub struct ManageEditSessionService<R, S, M>
where
    R: ProjectEditingSessionRepository<Error = ProjectRepositoryError> + Send + Sync + 'static,
    S: ProjectSnapshotRepository<Error = ProjectRepositoryError> + Send + Sync + 'static,
    M: RedisDataManager<Error = FlowProjectRedisDataManagerError> + Send + Sync + 'static,
{
    pub session_repository: Arc<R>,
    pub snapshot_repository: Arc<S>,
    pub redis_data_manager: Arc<M>,
}

#[automock]
impl<R, S, M> ManageEditSessionService<R, S, M>
where
    R: ProjectEditingSessionRepository<Error = ProjectRepositoryError> + Send + Sync + 'static,
    S: ProjectSnapshotRepository<Error = ProjectRepositoryError> + Send + Sync + 'static,
    M: RedisDataManager<Error = FlowProjectRedisDataManagerError> + Send + Sync + 'static,
{
    pub fn new(
        session_repository: Arc<R>,
        snapshot_repository: Arc<S>,
        redis_data_manager: Arc<M>,
    ) -> Self {
        Self {
            session_repository,
            snapshot_repository,
            redis_data_manager,
        }
    }

    pub async fn process(
        &self,
        mut data: ManageProjectEditSessionTaskData,
    ) -> Result<(), ProjectServiceError> {
        if let Some(mut session) = self
            .session_repository
            .get_active_session(&data.project_id)
            .await?
        {
            debug!(session = ?session, "Active session found");

            session
                .load_session(&*self.snapshot_repository, &data.session_id)
                .await?;

            debug!(session = ?session, "Session after load_session");

            let client_count = self.update_client_count(&mut data).await?;
            debug!(client_count = client_count, "Updated client count");

            self.merge_updates(&mut session, &mut data).await?;
            debug!("Updates merged");

            self.create_snapshot_if_required(&mut session, &mut data)
                .await?;
            debug!("Snapshot created if required");

            let session_ended = self
                .end_editing_session_if_conditions_met(&mut session, &data, client_count)
                .await?;
            debug!(session_ended = session_ended, "Session end check completed");

            if !session_ended {
                self.complete_job_if_met_requirements(&session, &data)
                    .await?;
                debug!("Job completion check completed");

                self.session_repository.update_session(session).await?;
                debug!("Session updated");
            } else {
                debug!("Session ended, skipping further processing");
            }
        } else {
            debug!("No active session found");
        }

        Ok(())
    }

    async fn update_client_count(
        &self,
        data: &mut ManageProjectEditSessionTaskData,
    ) -> Result<usize, ProjectServiceError> {
        let current_client_count = self.session_repository.get_client_count().await?;
        let old_client_count = data.clients_count.unwrap_or(0);
        data.clients_count = Some(current_client_count);

        if current_client_count == 0
            && old_client_count != current_client_count
            && data.clients_disconnected_at.is_none()
        {
            data.clients_disconnected_at = Some(Utc::now());
        } else if current_client_count > 0 {
            data.clients_disconnected_at = None;
        }

        Ok(current_client_count)
    }

    async fn merge_updates(
        &self,
        session: &mut ProjectEditingSession,
        data: &mut ManageProjectEditSessionTaskData,
    ) -> Result<(), ProjectServiceError> {
        session.merge_updates(&*self.redis_data_manager).await?;
        data.last_merged_at = Some(Utc::now());
        Ok(())
    }

    async fn create_snapshot_if_required(
        &self,
        session: &mut ProjectEditingSession,
        data: &mut ManageProjectEditSessionTaskData,
    ) -> Result<(), ProjectServiceError> {
        let current_time = Utc::now();
        let should_create_snapshot = match data.last_snapshot_at {
            Some(last_snapshot_at) => {
                (current_time - last_snapshot_at).num_milliseconds()
                    > MAX_SNAPSHOT_DELTA.as_millis() as i64
            }
            None => true, // Create a snapshot if there's no previous snapshot
        };

        if should_create_snapshot {
            self.create_snapshot(session, current_time).await?;
            data.last_snapshot_at = Some(current_time);
        }
        Ok(())
    }

    async fn create_snapshot(
        &self,
        session: &mut ProjectEditingSession,
        current_time: DateTime<Utc>,
    ) -> Result<(), ProjectServiceError> {
        let state = session.get_state_update(&*self.redis_data_manager).await?;

        let metadata = Metadata::new(
            generate_id(14, "snap"),
            session.project_id.clone(),
            session.session_id.clone(),
            String::new(),
            String::new(),
        );

        let snapshot_state = SnapshotInfo::new(
            None,
            vec![],
            ObjectTenant {
                id: session.tenant.id.clone(),
                key: session.tenant.key.clone(),
            },
            ObjectDelete {
                deleted: false,
                delete_after: None,
            },
            Some(current_time),
            Some(current_time),
        );

        let snapshot = ProjectSnapshot::new(metadata, snapshot_state);
        let snapshot_data = SnapshotData::new(session.project_id.clone(), state, None, None);

        self.snapshot_repository.create_snapshot(snapshot).await?;
        self.snapshot_repository
            .create_snapshot_state(snapshot_data)
            .await?;

        Ok(())
    }

    async fn end_editing_session_if_conditions_met(
        &self,
        session: &mut ProjectEditingSession,
        data: &ManageProjectEditSessionTaskData,
        client_count: usize,
    ) -> Result<bool, ProjectServiceError> {
        debug!(
            session = ?session,
            client_count = client_count,
            "Entering end_editing_session_if_conditions_met"
        );

        if let Some(clients_disconnected_at) = data.clients_disconnected_at {
            let current_time = Utc::now();
            let clients_disconnection_elapsed_time = current_time - clients_disconnected_at;

            debug!(
                clients_disconnected_at = ?clients_disconnected_at,
                current_time = ?current_time,
                disconnection_elapsed_time = ?clients_disconnection_elapsed_time,
                "Checking session end conditions"
            );

            if clients_disconnection_elapsed_time
                .to_std()
                .map_err(ProjectServiceError::ChronoDurationConversionError)?
                > MAX_EMPTY_SESSION_DURATION
                && client_count == 0
            {
                debug!("Conditions met for ending session");
                if session.session_setup_complete {
                    match session
                        .end_session(&*self.redis_data_manager, &*self.snapshot_repository)
                        .await
                    {
                        Ok(_) => {
                            debug!("Session ended successfully");
                            return Ok(true);
                        }
                        Err(e) => {
                            debug!(error = ?e, "Error ending session");
                            return Err(ProjectServiceError::EditingSession(e));
                        }
                    }
                } else {
                    debug!("Session not setup, cannot end");
                    return Err(ProjectServiceError::EditingSession(
                        flow_websocket_domain::project::ProjectEditingSessionError::SessionNotSetup,
                    ));
                }
            }
        }

        Ok(false)
    }

    async fn complete_job_if_met_requirements(
        &self,
        session: &ProjectEditingSession,
        data: &ManageProjectEditSessionTaskData,
    ) -> Result<(), ProjectServiceError> {
        if session.active_editing_session().await?.as_ref() == Some(&data.session_id) {
            sleep(JOB_COMPLETION_DELAY).await;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};
    use mockall::predicate::*;

    mockall::mock! {
        ProjectEditingSessionRepository {}
        #[async_trait::async_trait]
        impl ProjectEditingSessionRepository for ProjectEditingSessionRepository {
            type Error = ProjectRepositoryError;

            async fn create_session(&self, session: ProjectEditingSession) -> Result<String, ProjectRepositoryError >;
            async fn get_active_session(&self, project_id: &str) -> Result<Option<ProjectEditingSession>, ProjectRepositoryError>;
            async fn update_session(&self, session: ProjectEditingSession) -> Result<(), ProjectRepositoryError>;
            async fn get_client_count(&self) -> Result<usize, ProjectRepositoryError>;
        }
    }

    mockall::mock! {
    ProjectSnapshotRepository {}
    #[async_trait::async_trait]
    impl ProjectSnapshotRepository for ProjectSnapshotRepository {
        type Error = ProjectRepositoryError;

        async fn create_snapshot(&self, snapshot: ProjectSnapshot) -> Result<(), ProjectRepositoryError>;
        async fn get_latest_snapshot(&self, project_id: &str) -> Result<Option<ProjectSnapshot>, ProjectRepositoryError>;
        async fn get_latest_snapshot_state(&self, project_id: &str) -> Result<Vec<u8>, ProjectRepositoryError>;
        async fn update_latest_snapshot(&self, snapshot: ProjectSnapshot) -> Result<(), ProjectRepositoryError>;
        async fn update_latest_snapshot_state(&self, project_id: &str, snapshot_data: SnapshotData) -> Result<(), ProjectRepositoryError>;
        async fn delete_snapshot_state(&self, project_id: &str) -> Result<(), ProjectRepositoryError>;
        async fn create_snapshot_state(&self, snapshot_data: SnapshotData) -> Result<(), ProjectRepositoryError>;

    }
    }

    mockall::mock! {
        RedisDataManager {}
        #[async_trait::async_trait]
        impl RedisDataManager for RedisDataManager {
            type Error = FlowProjectRedisDataManagerError;

            async fn merge_updates(&self, skip_lock: bool) -> Result<(Vec<u8>, Vec<String>), FlowProjectRedisDataManagerError>;
            async fn get_current_state(&self) -> Result<Option<Vec<u8>>, FlowProjectRedisDataManagerError>;
            async fn push_update(&self, update: Vec<u8>, updated_by: Option<String>) -> Result<(), FlowProjectRedisDataManagerError>;
            async fn clear_data(&self) -> Result<(), FlowProjectRedisDataManagerError>;
        }
    }

    #[tokio::test]
    async fn test_process_with_active_session() {
        let (mut service, mut mocks) = setup_service();
        let task_data = create_task_data();

        let mut session = create_session("project_123");
        session.session_id = Some("session_456".to_string());

        mocks
            .session_repo
            .expect_get_active_session()
            .with(eq("project_123"))
            .times(1)
            .returning(move |_| Ok(Some(session.clone())));

        mocks
            .session_repo
            .expect_get_client_count()
            .times(1)
            .returning(|| Ok(1));

        mocks
            .redis_manager
            .expect_merge_updates()
            .with(eq(false))
            .times(1)
            .returning(|_| Ok((vec![], vec![])));

        mocks
            .session_repo
            .expect_update_session()
            .times(1)
            .returning(|_| Ok(()));

        mocks
            .snapshot_repo
            .expect_get_latest_snapshot()
            .with(eq("session_456"))
            .times(1)
            .returning(|_| Ok(Some(create_project_snapshot())));

        mocks
            .snapshot_repo
            .expect_get_latest_snapshot()
            .with(eq("project_123"))
            .times(..=1)
            .returning(|_| Ok(Some(create_project_snapshot())));

        service.session_repository = Arc::new(mocks.session_repo);
        service.snapshot_repository = Arc::new(mocks.snapshot_repo);
        service.redis_data_manager = Arc::new(mocks.redis_manager);

        let result = service.process(task_data).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_process_with_no_active_session() {
        let (mut service, mut mocks) = setup_service();
        let task_data = create_task_data();

        mocks
            .session_repo
            .expect_get_active_session()
            .with(eq("project_123"))
            .times(1)
            .returning(|_| Ok(None));

        service.session_repository = Arc::new(mocks.session_repo);

        let result = service.process(task_data).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_client_count() {
        let (mut service, mut mocks) = setup_service();
        let mut task_data = create_task_data();

        mocks
            .session_repo
            .expect_get_client_count()
            .times(1)
            .returning(|| Ok(0));

        service.session_repository = Arc::new(mocks.session_repo);

        let result = service.update_client_count(&mut task_data).await;
        assert!(result.is_ok());
        assert_eq!(task_data.clients_count, Some(0));
        assert!(task_data.clients_disconnected_at.is_some());
    }

    #[tokio::test]
    async fn test_merge_updates() {
        let (mut service, mut mocks) = setup_service();
        let mut task_data = create_task_data();
        let mut session = create_session("project_123");

        session.session_setup_complete = true;
        session.session_id = Some("session_456".to_string());

        mocks
            .redis_manager
            .expect_merge_updates()
            .with(eq(false))
            .times(1)
            .returning(|_| Ok((vec![1, 2, 3], vec!["user1".to_string()])));

        service.redis_data_manager = Arc::new(mocks.redis_manager);

        let result = service.merge_updates(&mut session, &mut task_data).await;

        if let Err(ref e) = result {
            println!("Error in merge_updates: {:?}", e);
        }

        assert!(result.is_ok());
        assert!(task_data.last_merged_at.is_some());
    }

    #[tokio::test]
    async fn test_create_snapshot_if_required() {
        let (mut service, mut mocks) = setup_service();
        let mut task_data = create_task_data();
        task_data.last_snapshot_at = Some(Utc::now() - Duration::minutes(6));
        let mut session = create_session("project_123");
        session.session_id = Some("session_456".to_string());
        session.session_setup_complete = true;

        mocks
            .redis_manager
            .expect_get_current_state()
            .times(1)
            .returning(|| Ok(Some(vec![1, 2, 3])));

        mocks
            .snapshot_repo
            .expect_create_snapshot()
            .times(1)
            .returning(|snapshot: ProjectSnapshot| {
                assert_eq!(snapshot.metadata.project_id, "project_123");
                assert_eq!(
                    snapshot.metadata.session_id,
                    Some("session_456".to_string())
                );
                Ok(())
            });

        mocks
            .snapshot_repo
            .expect_create_snapshot_state()
            .times(1)
            .returning(|snapshot_data: SnapshotData| {
                assert_eq!(snapshot_data.project_id, "project_123");
                assert_eq!(snapshot_data.state, vec![1, 2, 3]);
                Ok(())
            });

        service.redis_data_manager = Arc::new(mocks.redis_manager);
        service.snapshot_repository = Arc::new(mocks.snapshot_repo);

        let result = service
            .create_snapshot_if_required(&mut session, &mut task_data)
            .await;

        if let Err(ref e) = result {
            println!("Error in create_snapshot_if_required: {:?}", e);
        }

        assert!(result.is_ok());
        assert!(task_data.last_snapshot_at.unwrap() > Utc::now() - Duration::seconds(1));
    }

    #[tokio::test]
    async fn test_end_editing_session_if_conditions_met() {
        let (mut service, mut mocks) = setup_service();
        let mut task_data = create_task_data();
        task_data.clients_disconnected_at = Some(Utc::now() - Duration::seconds(11));
        let mut session = create_session("project_123");
        session.session_id = Some("session_456".to_string());
        session.session_setup_complete = true;

        mocks
            .redis_manager
            .expect_merge_updates()
            .with(eq(true))
            .times(1)
            .returning(|_| Ok((vec![1, 2, 3], vec!["user1".to_string()])));

        mocks
            .snapshot_repo
            .expect_update_latest_snapshot_state()
            .times(1)
            .returning(|_, _| Ok(()));

        mocks
            .redis_manager
            .expect_clear_data()
            .times(1)
            .returning(|| Ok(()));

        service.session_repository = Arc::new(mocks.session_repo);
        service.redis_data_manager = Arc::new(mocks.redis_manager);
        service.snapshot_repository = Arc::new(mocks.snapshot_repo);

        let result = service
            .end_editing_session_if_conditions_met(&mut session, &task_data, 0)
            .await;
        assert!(result.is_ok());
        assert!(session.session_id.is_none());
        assert!(!session.session_setup_complete);
    }

    #[tokio::test]
    async fn test_complete_job_if_met_requirements() {
        let (service, _mocks) = setup_service();
        let task_data = create_task_data();
        let mut session = create_session("project_123");

        session.session_setup_complete = true;
        session.session_id = Some("session_456".to_string());

        let result = service
            .complete_job_if_met_requirements(&session, &task_data)
            .await;

        if let Err(ref e) = result {
            println!("Error: {:?}", e);
        }

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_process_with_snapshot_creation() {
        let (mut service, mut mocks) = setup_service();
        let mut task_data = create_task_data();
        task_data.last_snapshot_at = Some(Utc::now() - Duration::minutes(10));

        let mut session = create_session("project_123");
        session.session_id = Some("session_456".to_string());

        mocks
            .session_repo
            .expect_get_active_session()
            .with(eq("project_123"))
            .times(1)
            .returning(move |_| Ok(Some(session.clone())));

        mocks
            .session_repo
            .expect_get_client_count()
            .times(1)
            .returning(|| Ok(1));

        mocks
            .redis_manager
            .expect_merge_updates()
            .with(eq(false))
            .times(1)
            .returning(|_| Ok((vec![], vec![])));

        mocks
            .redis_manager
            .expect_get_current_state()
            .times(1)
            .returning(|| Ok(Some(vec![1, 2, 3])));

        mocks
            .snapshot_repo
            .expect_create_snapshot()
            .times(1)
            .returning(|_| Ok(()));

        mocks
            .snapshot_repo
            .expect_create_snapshot_state()
            .times(1)
            .returning(|_| Ok(()));

        mocks
            .session_repo
            .expect_update_session()
            .times(1)
            .returning(|_| Ok(()));

        mocks
            .snapshot_repo
            .expect_get_latest_snapshot()
            .with(eq("session_456"))
            .times(1)
            .returning(|_| Ok(Some(create_project_snapshot())));

        mocks
            .snapshot_repo
            .expect_get_latest_snapshot()
            .with(eq("project_123"))
            .times(..=1)
            .returning(|_| Ok(Some(create_project_snapshot())));

        service.session_repository = Arc::new(mocks.session_repo);
        service.snapshot_repository = Arc::new(mocks.snapshot_repo);
        service.redis_data_manager = Arc::new(mocks.redis_manager);

        let result = service.process(task_data).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_process_with_session_end() {
        let (mut service, mut mocks) = setup_service();
        let mut task_data = create_task_data();
        task_data.clients_count = Some(0);
        task_data.clients_disconnected_at = Some(Utc::now() - Duration::seconds(15));

        let mut session = create_session("project_123");
        session.session_id = Some("session_456".to_string());
        session.session_setup_complete = true;

        mocks
            .session_repo
            .expect_get_active_session()
            .with(eq("project_123"))
            .times(1)
            .returning(move |_| Ok(Some(session.clone())));

        mocks
            .session_repo
            .expect_get_client_count()
            .times(1)
            .returning(|| Ok(0));

        mocks
            .redis_manager
            .expect_merge_updates()
            .with(eq(false))
            .times(1)
            .returning(|_| Ok((vec![], vec![])));

        mocks
            .redis_manager
            .expect_merge_updates()
            .with(eq(true))
            .times(1)
            .returning(|_| Ok((vec![], vec![])));

        mocks
            .snapshot_repo
            .expect_update_latest_snapshot_state()
            .times(1)
            .returning(|_, _| Ok(()));

        mocks
            .redis_manager
            .expect_clear_data()
            .times(1)
            .returning(|| Ok(()));

        mocks
            .snapshot_repo
            .expect_get_latest_snapshot()
            .with(eq("session_456"))
            .times(1)
            .returning(|_| Ok(Some(create_project_snapshot())));

        mocks
            .snapshot_repo
            .expect_get_latest_snapshot()
            .with(eq("project_123"))
            .times(..=1)
            .returning(|_| Ok(Some(create_project_snapshot())));

        service.session_repository = Arc::new(mocks.session_repo);
        service.snapshot_repository = Arc::new(mocks.snapshot_repo);
        service.redis_data_manager = Arc::new(mocks.redis_manager);

        let result = service.process(task_data).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_process_with_error() {
        let (mut service, mut mocks) = setup_service();
        let task_data = create_task_data();

        mocks
            .session_repo
            .expect_get_active_session()
            .with(eq("project_123"))
            .times(1)
            .returning(|_| Err(ProjectRepositoryError::Custom("Test error".to_string())));

        service.session_repository = Arc::new(mocks.session_repo);

        let result = service.process(task_data).await;
        assert!(result.is_err());
    }

    // Helper functions

    fn setup_service() -> (
        ManageEditSessionService<
            MockProjectEditingSessionRepository,
            MockProjectSnapshotRepository,
            MockRedisDataManager,
        >,
        MockRepositories,
    ) {
        let mock_session_repo = MockProjectEditingSessionRepository::new();
        let mock_snapshot_repo = MockProjectSnapshotRepository::new();
        let mock_redis_manager = MockRedisDataManager::new();

        let service = ManageEditSessionService::new(
            Arc::new(MockProjectEditingSessionRepository::new()),
            Arc::new(MockProjectSnapshotRepository::new()),
            Arc::new(MockRedisDataManager::new()),
        );

        let mocks = MockRepositories {
            session_repo: mock_session_repo,
            snapshot_repo: mock_snapshot_repo,
            redis_manager: mock_redis_manager,
        };

        (service, mocks)
    }

    fn create_task_data() -> ManageProjectEditSessionTaskData {
        ManageProjectEditSessionTaskData {
            project_id: "project_123".to_string(),
            session_id: "session_456".to_string(),
            clients_count: Some(1),
            clients_disconnected_at: None,
            last_merged_at: None,
            last_snapshot_at: Some(Utc::now() - Duration::minutes(4)),
        }
    }

    fn create_session(project_id: &str) -> ProjectEditingSession {
        let mut session = ProjectEditingSession::new(
            project_id.to_string(),
            ObjectTenant::new(generate_id(14, "tenant"), "tenant".to_owned()),
        );
        session.session_setup_complete = true;
        session.session_id = Some(generate_id(14, "session"));
        session
    }

    fn create_project_snapshot() -> ProjectSnapshot {
        ProjectSnapshot::new(
            Metadata::new(
                generate_id(14, "snap"),
                "project_123".to_string(),
                Some("session_456".to_string()),
                "Test Snapshot".to_string(),
                "".to_string(),
            ),
            SnapshotInfo::new(
                Some("test_user".to_string()),
                vec![],
                ObjectTenant::new(generate_id(14, "tenant"), "tenant".to_owned()),
                ObjectDelete {
                    deleted: false,
                    delete_after: None,
                },
                Some(Utc::now()),
                None,
            ),
        )
    }

    struct MockRepositories {
        session_repo: MockProjectEditingSessionRepository,
        snapshot_repo: MockProjectSnapshotRepository,
        redis_manager: MockRedisDataManager,
    }
}
