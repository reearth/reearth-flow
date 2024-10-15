use chrono::{DateTime, Utc};
use flow_websocket_domain::{
    generate_id,
    project::ProjectEditingSession,
    repository::{
        ProjectEditingSessionRepository, ProjectSnapshotRepository, RedisDataManager,
        SnapshotDataRepository,
    },
    snapshot::{Metadata, ObjectDelete, ObjectTenant, SnapshotInfo},
    types::{data::SnapshotData, snapshot::ProjectSnapshot},
};
use mockall::automock;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

use crate::{types::ManageProjectEditSessionTaskData, ProjectServiceError};

const MAX_EMPTY_SESSION_DURATION: Duration = Duration::from_secs(10);
const MAX_SNAPSHOT_DELTA: Duration = Duration::from_secs(5 * 60);
const JOB_COMPLETION_DELAY: Duration = Duration::from_secs(5);

pub struct ManageEditSessionService<R, S, D, M>
where
    R: ProjectEditingSessionRepository<Error = ProjectServiceError> + Send + Sync,
    S: ProjectSnapshotRepository<Error = ProjectServiceError> + Send + Sync,
    D: SnapshotDataRepository<Error = ProjectServiceError> + Send + Sync,
    M: RedisDataManager<Error = ProjectServiceError> + Send + Sync,
{
    pub session_repository: Arc<R>,
    pub snapshot_repository: Arc<S>,
    pub snapshot_data_repository: Arc<D>,
    pub redis_data_manager: Arc<M>,
}

#[automock]
impl<R, S, D, M> ManageEditSessionService<R, S, D, M>
where
    R: ProjectEditingSessionRepository<Error = ProjectServiceError> + Send + Sync + 'static,
    S: ProjectSnapshotRepository<Error = ProjectServiceError> + Send + Sync + 'static,
    D: SnapshotDataRepository<Error = ProjectServiceError> + Send + Sync + 'static,
    M: RedisDataManager<Error = ProjectServiceError> + Send + Sync + 'static,
{
    pub fn new(
        session_repository: Arc<R>,
        snapshot_repository: Arc<S>,
        snapshot_data_repository: Arc<D>,
        redis_data_manager: Arc<M>,
    ) -> Self {
        Self {
            session_repository,
            snapshot_repository,
            snapshot_data_repository,
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
            session
                .load_session(&*self.snapshot_repository, &data.session_id)
                .await?;

            let client_count = self.update_client_count(&mut data).await?;
            self.merge_updates(&mut session, &mut data).await?;
            self.create_snapshot_if_required(&mut session, &mut data)
                .await?;
            self.end_editing_session_if_conditions_met(&mut session, &data, client_count)
                .await?;
            self.complete_job_if_met_requirements(&session, &data)
                .await?;

            self.session_repository.update_session(session).await?;
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
        session
            .merge_updates(&*self.redis_data_manager, false)
            .await?;
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
        self.snapshot_data_repository
            .create_snapshot_data(snapshot_data)
            .await?;

        Ok(())
    }

    async fn end_editing_session_if_conditions_met(
        &self,
        session: &mut ProjectEditingSession,
        data: &ManageProjectEditSessionTaskData,
        client_count: usize,
    ) -> Result<(), ProjectServiceError> {
        if let Some(clients_disconnected_at) = data.clients_disconnected_at {
            let current_time = Utc::now();
            let clients_disconnection_elapsed_time = current_time - clients_disconnected_at;

            if clients_disconnection_elapsed_time
                .to_std()
                .map_err(ProjectServiceError::ChronoDurationConversionError)?
                > MAX_EMPTY_SESSION_DURATION
                && client_count == 0
            {
                session
                    .end_session(&*self.redis_data_manager, &*self.snapshot_repository)
                    .await?;
            }
        }
        Ok(())
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
            type Error = ProjectServiceError;

            async fn create_session(&self, session: ProjectEditingSession) -> Result<(), ProjectServiceError>;
            async fn get_active_session(&self, project_id: &str) -> Result<Option<ProjectEditingSession>, ProjectServiceError>;
            async fn update_session(&self, session: ProjectEditingSession) -> Result<(), ProjectServiceError>;
            async fn get_client_count(&self) -> Result<usize, ProjectServiceError>;
        }
    }

    mockall::mock! {
        ProjectSnapshotRepository {}
        #[async_trait::async_trait]
        impl ProjectSnapshotRepository for ProjectSnapshotRepository {
            type Error = ProjectServiceError;

            async fn create_snapshot(&self, snapshot: ProjectSnapshot) -> Result<(), ProjectServiceError>;
            async fn get_latest_snapshot(&self, project_id: &str) -> Result<Option<ProjectSnapshot>, ProjectServiceError>;
            async fn get_latest_snapshot_state(&self, project_id: &str) -> Result<Vec<u8>, ProjectServiceError>;
            async fn update_latest_snapshot(&self, snapshot: ProjectSnapshot) -> Result<(), ProjectServiceError>;
            async fn update_latest_snapshot_data(&self, project_id: &str, snapshot_data: SnapshotData) -> Result<(), ProjectServiceError>;
        }
    }

    mockall::mock! {
        SnapshotDataRepository {}
        #[async_trait::async_trait]
        impl SnapshotDataRepository for SnapshotDataRepository {
            type Error = ProjectServiceError;

            async fn create_snapshot_data(&self, snapshot_data: SnapshotData) -> Result<(), ProjectServiceError>;
            async fn get_snapshot_data(&self, project_id: &str) -> Result<Option<Vec<u8>>, ProjectServiceError>;
            async fn get_latest_snapshot_data(&self, project_id: &str) -> Result<Option<Vec<u8>>, ProjectServiceError>;
            async fn update_latest_snapshot_data(&self, project_id: &str, snapshot_data: SnapshotData) -> Result<(), ProjectServiceError>;
            async fn delete_snapshot_data(&self, project_id: &str) -> Result<(), ProjectServiceError>;
        }
    }

    mockall::mock! {
        RedisDataManager {}
        #[async_trait::async_trait]
        impl RedisDataManager for RedisDataManager {
            type Error = ProjectServiceError;

            async fn merge_updates(&self, skip_lock: bool) -> Result<(Vec<u8>, Vec<String>), ProjectServiceError>;
            async fn get_current_state(&self) -> Result<Option<Vec<u8>>, ProjectServiceError>;
            async fn push_update(&self, update: Vec<u8>, updated_by: Option<String>) -> Result<(), ProjectServiceError>;
            async fn clear_data(&self) -> Result<(), ProjectServiceError>;
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
        service.snapshot_data_repository = Arc::new(mocks.snapshot_data_repo);
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
            .snapshot_data_repo
            .expect_create_snapshot_data()
            .times(1)
            .returning(|snapshot_data: SnapshotData| {
                assert_eq!(snapshot_data.project_id, "project_123");
                assert_eq!(snapshot_data.state, vec![1, 2, 3]);
                Ok(())
            });

        service.redis_data_manager = Arc::new(mocks.redis_manager);
        service.snapshot_repository = Arc::new(mocks.snapshot_repo);
        service.snapshot_data_repository = Arc::new(mocks.snapshot_data_repo);

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
            .expect_update_latest_snapshot_data()
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

    // Helper functions

    fn setup_service() -> (
        ManageEditSessionService<
            MockProjectEditingSessionRepository,
            MockProjectSnapshotRepository,
            MockSnapshotDataRepository,
            MockRedisDataManager,
        >,
        MockRepositories,
    ) {
        let mock_session_repo = MockProjectEditingSessionRepository::new();
        let mock_snapshot_repo = MockProjectSnapshotRepository::new();
        let mock_snapshot_data_repo = MockSnapshotDataRepository::new();
        let mock_redis_manager = MockRedisDataManager::new();

        let service = ManageEditSessionService::new(
            Arc::new(MockProjectEditingSessionRepository::new()),
            Arc::new(MockProjectSnapshotRepository::new()),
            Arc::new(MockSnapshotDataRepository::new()),
            Arc::new(MockRedisDataManager::new()),
        );

        let mocks = MockRepositories {
            session_repo: mock_session_repo,
            snapshot_repo: mock_snapshot_repo,
            snapshot_data_repo: mock_snapshot_data_repo,
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
        snapshot_data_repo: MockSnapshotDataRepository,
        redis_manager: MockRedisDataManager,
    }
}
