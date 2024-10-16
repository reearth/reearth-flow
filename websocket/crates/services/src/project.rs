use crate::error::ProjectServiceError;
use async_trait::async_trait;
use flow_websocket_domain::generate_id;
use flow_websocket_domain::project::ProjectEditingSession;
use flow_websocket_domain::project_type::{Action, Project, ProjectAllowedActions};
use flow_websocket_domain::repository::{
    ProjectEditingSessionRepository, ProjectRepository, ProjectSnapshotRepository, RedisDataManager,
};
use flow_websocket_domain::snapshot::{ObjectTenant, ProjectSnapshot};
use flow_websocket_domain::types::data::SnapshotData;
use flow_websocket_infra::persistence::project_repository::ProjectRepositoryError;
use std::sync::Arc;

pub struct ProjectService<E, S, R> {
    session_repository: Arc<E>,
    snapshot_repository: Arc<S>,
    redis_data_manager: Arc<R>,
}

impl<E, S, R> ProjectService<E, S, R>
where
    E: ProjectEditingSessionRepository<Error = ProjectRepositoryError>
        + ProjectRepository<Error = ProjectRepositoryError>
        + Send
        + Sync,
    S: ProjectSnapshotRepository<Error = ProjectRepositoryError> + Send + Sync,
    R: RedisDataManager<Error = ProjectRepositoryError> + Send + Sync,
{
    pub fn new(
        session_repository: Arc<E>,
        snapshot_repository: Arc<S>,
        redis_data_manager: Arc<R>,
    ) -> Self {
        Self {
            session_repository,
            snapshot_repository,
            redis_data_manager,
        }
    }

    pub async fn get_project(
        &self,
        project_id: &str,
    ) -> Result<Option<Project>, ProjectServiceError> {
        Ok(self.session_repository.get_project(project_id).await?)
    }

    pub async fn get_or_create_editing_session(
        &self,
        project_id: &str,
    ) -> Result<ProjectEditingSession, ProjectServiceError> {
        let mut session = match self
            .session_repository
            .get_active_session(project_id)
            .await?
        {
            Some(session) => session,

            None => ProjectEditingSession::new(
                project_id.to_string(),
                ObjectTenant::new(generate_id(14, "tenant"), "tenant".to_owned()),
            ),
        };

        if session.session_id.is_none() {
            session
                .start_or_join_session(&*self.snapshot_repository, &*self.redis_data_manager)
                .await?;
            self.session_repository
                .create_session(session.clone())
                .await?;
        }

        Ok(session)
    }

    pub async fn get_project_allowed_actions(
        &self,
        project_id: &str,
        actions: Vec<String>,
    ) -> Result<ProjectAllowedActions, ProjectServiceError> {
        Ok(ProjectAllowedActions {
            id: project_id.to_string(),
            actions: actions
                .into_iter()
                .map(|action| Action {
                    action,
                    allowed: true,
                })
                .collect(),
        })
    }
}

#[async_trait]
impl<E, S, R> ProjectRepository for ProjectService<E, S, R>
where
    E: ProjectEditingSessionRepository<Error = ProjectRepositoryError>
        + ProjectRepository<Error = ProjectRepositoryError>
        + Send
        + Sync,
    S: ProjectSnapshotRepository<Error = ProjectRepositoryError> + Send + Sync,
    R: RedisDataManager<Error = ProjectRepositoryError> + Send + Sync,
{
    type Error = ProjectServiceError;

    async fn get_project(&self, project_id: &str) -> Result<Option<Project>, Self::Error> {
        Ok(self.session_repository.get_project(project_id).await?)
    }
}

#[async_trait]
impl<E, S, R> ProjectEditingSessionRepository for ProjectService<E, S, R>
where
    E: ProjectEditingSessionRepository<Error = ProjectRepositoryError> + Send + Sync,
    S: ProjectSnapshotRepository<Error = ProjectRepositoryError> + Send + Sync,
    R: RedisDataManager<Error = ProjectRepositoryError> + Send + Sync,
{
    type Error = ProjectServiceError;

    async fn create_session(&self, session: ProjectEditingSession) -> Result<(), Self::Error> {
        Ok(self.session_repository.create_session(session).await?)
    }

    async fn get_active_session(
        &self,
        project_id: &str,
    ) -> Result<Option<ProjectEditingSession>, Self::Error> {
        Ok(self
            .session_repository
            .get_active_session(project_id)
            .await?)
    }

    async fn update_session(&self, session: ProjectEditingSession) -> Result<(), Self::Error> {
        Ok(self.session_repository.update_session(session).await?)
    }

    async fn get_client_count(&self) -> Result<usize, Self::Error> {
        Ok(self.session_repository.get_client_count().await?)
    }
}

#[async_trait]
impl<E, S, R> ProjectSnapshotRepository for ProjectService<E, S, R>
where
    E: ProjectEditingSessionRepository<Error = ProjectRepositoryError>
        + ProjectRepository<Error = ProjectRepositoryError>
        + Send
        + Sync,
    S: ProjectSnapshotRepository<Error = ProjectRepositoryError> + Send + Sync,
    R: RedisDataManager<Error = ProjectRepositoryError> + Send + Sync,
{
    type Error = ProjectServiceError;

    async fn create_snapshot(&self, snapshot: ProjectSnapshot) -> Result<(), Self::Error> {
        Ok(self.snapshot_repository.create_snapshot(snapshot).await?)
    }

    async fn create_snapshot_state(&self, snapshot_data: SnapshotData) -> Result<(), Self::Error> {
        Ok(self
            .snapshot_repository
            .create_snapshot_state(snapshot_data)
            .await?)
    }

    async fn get_latest_snapshot(
        &self,
        project_id: &str,
    ) -> Result<Option<ProjectSnapshot>, Self::Error> {
        Ok(self
            .snapshot_repository
            .get_latest_snapshot(project_id)
            .await?)
    }

    async fn get_latest_snapshot_state(&self, project_id: &str) -> Result<Vec<u8>, Self::Error> {
        Ok(self
            .snapshot_repository
            .get_latest_snapshot_state(project_id)
            .await?)
    }

    async fn update_latest_snapshot(&self, snapshot: ProjectSnapshot) -> Result<(), Self::Error> {
        Ok(self
            .snapshot_repository
            .update_latest_snapshot(snapshot)
            .await?)
    }

    async fn update_latest_snapshot_state(
        &self,
        project_id: &str,
        snapshot_data: SnapshotData,
    ) -> Result<(), Self::Error> {
        Ok(self
            .snapshot_repository
            .update_latest_snapshot_state(project_id, snapshot_data)
            .await?)
    }

    async fn delete_snapshot_state(&self, project_id: &str) -> Result<(), Self::Error> {
        Ok(self
            .snapshot_repository
            .delete_snapshot_state(project_id)
            .await?)
    }
}

#[async_trait]
impl<E, S, R> RedisDataManager for ProjectService<E, S, R>
where
    E: ProjectEditingSessionRepository<Error = ProjectRepositoryError>
        + ProjectRepository<Error = ProjectRepositoryError>
        + Send
        + Sync,
    S: ProjectSnapshotRepository<Error = ProjectRepositoryError> + Send + Sync,
    R: RedisDataManager<Error = ProjectRepositoryError> + Send + Sync,
{
    type Error = ProjectServiceError;

    async fn push_update(
        &self,
        update: Vec<u8>,
        updated_by: Option<String>,
    ) -> Result<(), Self::Error> {
        Ok(self
            .redis_data_manager
            .push_update(update, updated_by)
            .await?)
    }

    async fn merge_updates(&self, skip_lock: bool) -> Result<(Vec<u8>, Vec<String>), Self::Error> {
        Ok(self.redis_data_manager.merge_updates(skip_lock).await?)
    }

    async fn get_current_state(&self) -> Result<Option<Vec<u8>>, Self::Error> {
        Ok(self.redis_data_manager.get_current_state().await?)
    }

    async fn clear_data(&self) -> Result<(), Self::Error> {
        Ok(self.redis_data_manager.clear_data().await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flow_websocket_domain::snapshot::Metadata;
    use flow_websocket_domain::snapshot::ObjectDelete;
    use flow_websocket_domain::snapshot::SnapshotInfo;
    use mockall::mock;
    use mockall::predicate::*;

    mock! {
        SessionRepo {}
        #[async_trait]
        impl ProjectEditingSessionRepository for SessionRepo {
            type Error = ProjectRepositoryError;
            async fn create_session(&self, session: ProjectEditingSession) -> Result<(), ProjectRepositoryError>;
            async fn get_active_session(&self, project_id: &str) -> Result<Option<ProjectEditingSession>, ProjectRepositoryError>;
            async fn update_session(&self, session: ProjectEditingSession) -> Result<(), ProjectRepositoryError>;
            async fn get_client_count(&self) -> Result<usize, ProjectRepositoryError>;
        }
        #[async_trait]
        impl ProjectRepository for SessionRepo {
            type Error = ProjectRepositoryError;
            async fn get_project(&self, project_id: &str) -> Result<Option<Project>, ProjectRepositoryError>;
        }
    }

    mock! {
        SnapshotRepo {}
        #[async_trait]
        impl ProjectSnapshotRepository for SnapshotRepo {
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

    mock! {
        RedisManager {}
        #[async_trait]
        impl RedisDataManager for RedisManager {
            type Error = ProjectRepositoryError;
            async fn push_update(&self, update: Vec<u8>, updated_by: Option<String>) -> Result<(), ProjectRepositoryError>;
            async fn merge_updates(&self, skip_lock: bool) -> Result<(Vec<u8>, Vec<String>), ProjectRepositoryError>;
            async fn get_current_state(&self) -> Result<Option<Vec<u8>>, ProjectRepositoryError>;
            async fn clear_data(&self) -> Result<(), ProjectRepositoryError>;
        }
    }

    #[tokio::test]
    async fn test_merge_updates() {
        let mock_session_repo = MockSessionRepo::new();
        let mock_snapshot_repo = MockSnapshotRepo::new();
        let mut mock_redis_manager = MockRedisManager::new();

        // Set up expectations
        mock_redis_manager
            .expect_merge_updates()
            .with(eq(false))
            .times(1)
            .returning(|_| Ok((vec![1, 2, 3], vec!["user1".to_string()])));

        let service = ProjectService::new(
            Arc::new(mock_session_repo),
            Arc::new(mock_snapshot_repo),
            Arc::new(mock_redis_manager),
        );

        let result = service.merge_updates(false).await;

        assert!(result.is_ok());
        if let Ok((state, users)) = result {
            assert_eq!(state, vec![1, 2, 3]);
            assert_eq!(users, vec!["user1".to_string()]);
        }
    }

    #[tokio::test]
    async fn test_get_project() {
        let mut mock_session_repo = MockSessionRepo::new();
        let mock_snapshot_repo = MockSnapshotRepo::new();
        let mock_redis_manager = MockRedisManager::new();

        let example_project = Project {
            id: "project_123".to_string(),
            workspace_id: "workspace_456".to_string(),
        };

        mock_session_repo
            .expect_get_project()
            .with(eq("project_123"))
            .times(1)
            .returning(move |_| Ok(Some(example_project.clone())));

        let service = ProjectService::new(
            Arc::new(mock_session_repo),
            Arc::new(mock_snapshot_repo),
            Arc::new(mock_redis_manager),
        );

        let result = service.get_project("project_123").await;
        assert!(result.is_ok());
        let project = result.unwrap();
        assert!(project.is_some());
        let project = project.unwrap();
        assert_eq!(project.id, "project_123");
        assert_eq!(project.workspace_id, "workspace_456");
    }

    #[tokio::test]
    async fn test_get_or_create_editing_session() {
        let mut mock_session_repo = MockSessionRepo::new();
        let mut mock_snapshot_repo = MockSnapshotRepo::new();
        let mut mock_redis_manager = MockRedisManager::new();

        mock_session_repo
            .expect_get_active_session()
            .with(eq("project_123"))
            .times(1)
            .returning(|_| Ok(None));

        mock_session_repo
            .expect_create_session()
            .times(1)
            .returning(|_| Ok(()));

        mock_snapshot_repo
            .expect_get_latest_snapshot_state()
            .times(1)
            .returning(|_| Ok(vec![1, 2, 3]));

        mock_redis_manager
            .expect_push_update()
            .times(1)
            .returning(|_, _| Ok(()));

        let service = ProjectService::new(
            Arc::new(mock_session_repo),
            Arc::new(mock_snapshot_repo),
            Arc::new(mock_redis_manager),
        );

        let result = service.get_or_create_editing_session("project_123").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_project_allowed_actions() {
        let mock_session_repo = MockSessionRepo::new();
        let mock_snapshot_repo = MockSnapshotRepo::new();
        let mock_redis_manager = MockRedisManager::new();

        let service = ProjectService::new(
            Arc::new(mock_session_repo),
            Arc::new(mock_snapshot_repo),
            Arc::new(mock_redis_manager),
        );

        let actions = vec!["read".to_string(), "write".to_string()];
        let result = service
            .get_project_allowed_actions("project_123", actions)
            .await;

        assert!(result.is_ok());
        let allowed_actions = result.unwrap();
        assert_eq!(allowed_actions.id, "project_123");
        assert_eq!(allowed_actions.actions.len(), 2);
        assert!(allowed_actions.actions.iter().all(|a| a.allowed));
        assert_eq!(allowed_actions.actions[0].action, "read");
        assert_eq!(allowed_actions.actions[1].action, "write");
    }

    #[tokio::test]
    async fn test_create_snapshot() {
        let mock_session_repo = MockSessionRepo::new();
        let mut mock_snapshot_repo = MockSnapshotRepo::new();
        let mock_redis_manager = MockRedisManager::new();

        let snapshot = ProjectSnapshot::new(
            Metadata::new(
                "snapshot_123".to_string(),
                "project_123".to_string(),
                Some("session_456".to_string()),
                "New Snapshot".to_string(),
                "/path/to/new/snapshot".to_string(),
            ),
            SnapshotInfo::new(
                Some("user_789".to_string()),
                vec!["user_789".to_string()],
                ObjectTenant::new("tenant_123".to_string(), "tenant_key".to_string()),
                ObjectDelete {
                    deleted: false,
                    delete_after: None,
                },
                None,
                None,
            ),
        );

        mock_snapshot_repo
            .expect_create_snapshot()
            .with(function(|s: &ProjectSnapshot| {
                s.metadata.project_id == "project_123"
            }))
            .times(1)
            .returning(|_| Ok(()));

        let service = ProjectService::new(
            Arc::new(mock_session_repo),
            Arc::new(mock_snapshot_repo),
            Arc::new(mock_redis_manager),
        );

        let result = service.create_snapshot(snapshot).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_latest_snapshot() {
        let mock_session_repo = MockSessionRepo::new();
        let mut mock_snapshot_repo = MockSnapshotRepo::new();
        let mock_redis_manager = MockRedisManager::new();

        let example_snapshot = ProjectSnapshot::new(
            Metadata::new(
                "snapshot_123".to_string(),
                "project_123".to_string(),
                Some("session_456".to_string()),
                "Example Snapshot".to_string(),
                "/path/to/snapshot".to_string(),
            ),
            SnapshotInfo::new(
                Some("user_789".to_string()),
                vec!["user_789".to_string()],
                ObjectTenant::new("tenant_123".to_string(), "tenant_key".to_string()),
                ObjectDelete {
                    deleted: false,
                    delete_after: None,
                },
                None,
                None,
            ),
        );

        mock_snapshot_repo
            .expect_get_latest_snapshot()
            .with(eq("project_123"))
            .times(1)
            .returning(move |_| Ok(Some(example_snapshot.clone())));

        let service = ProjectService::new(
            Arc::new(mock_session_repo),
            Arc::new(mock_snapshot_repo),
            Arc::new(mock_redis_manager),
        );

        let result = service.get_latest_snapshot("project_123").await;
        assert!(result.is_ok());
        let snapshot = result.unwrap();
        assert!(snapshot.is_some());
        let snapshot = snapshot.unwrap();
        assert_eq!(snapshot.metadata.project_id, "project_123");
        assert_eq!(snapshot.metadata.id, "snapshot_123");
    }

    #[tokio::test]
    async fn test_push_update() {
        let mock_session_repo = MockSessionRepo::new();
        let mock_snapshot_repo = MockSnapshotRepo::new();
        let mut mock_redis_manager = MockRedisManager::new();

        mock_redis_manager
            .expect_push_update()
            .with(eq(vec![1, 2, 3]), eq(Some("user1".to_string())))
            .times(1)
            .returning(|_, _| Ok(()));

        let service = ProjectService::new(
            Arc::new(mock_session_repo),
            Arc::new(mock_snapshot_repo),
            Arc::new(mock_redis_manager),
        );

        let result = service
            .push_update(vec![1, 2, 3], Some("user1".to_string()))
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_current_state() {
        let mock_session_repo = MockSessionRepo::new();
        let mock_snapshot_repo = MockSnapshotRepo::new();
        let mut mock_redis_manager = MockRedisManager::new();

        mock_redis_manager
            .expect_get_current_state()
            .times(1)
            .returning(|| Ok(Some(vec![1, 2, 3])));

        let service = ProjectService::new(
            Arc::new(mock_session_repo),
            Arc::new(mock_snapshot_repo),
            Arc::new(mock_redis_manager),
        );

        let result = service.get_current_state().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(vec![1, 2, 3]));
    }

    #[tokio::test]
    async fn test_clear_data() {
        let mock_session_repo = MockSessionRepo::new();
        let mock_snapshot_repo = MockSnapshotRepo::new();
        let mut mock_redis_manager = MockRedisManager::new();

        mock_redis_manager
            .expect_clear_data()
            .times(1)
            .returning(|| Ok(()));

        let service = ProjectService::new(
            Arc::new(mock_session_repo),
            Arc::new(mock_snapshot_repo),
            Arc::new(mock_redis_manager),
        );

        let result = service.clear_data().await;
        assert!(result.is_ok());
    }
}
