use crate::error::ProjectServiceError;
use async_trait::async_trait;
use flow_websocket_domain::generate_id;
use flow_websocket_domain::project::ProjectEditingSession;
use flow_websocket_domain::projection::{Action, Project, ProjectAllowedActions};
use flow_websocket_domain::repository::{
    ProjectEditingSessionRepository, ProjectRepository, ProjectSnapshotRepository, RedisDataManager,
};
use flow_websocket_domain::snapshot::{ObjectTenant, ProjectSnapshot};
use flow_websocket_domain::types::data::SnapshotData;
use flow_websocket_infra::persistence::project_repository::ProjectRepositoryError;
use std::sync::Arc;

pub struct ProjectService<P, E, S, R> {
    project_repository: Arc<P>,
    session_repository: Arc<E>,
    snapshot_repository: Arc<S>,
    redis_data_manager: Arc<R>,
}

impl<P, E, S, R> ProjectService<P, E, S, R>
where
    P: ProjectRepository<Error = ProjectRepositoryError> + Send + Sync,
    E: ProjectEditingSessionRepository<Error = ProjectServiceError> + Send + Sync,
    S: ProjectSnapshotRepository<Error = ProjectRepositoryError> + Send + Sync,
    R: RedisDataManager<Error = ProjectRepositoryError> + Send + Sync,
{
    pub fn new(
        project_repository: Arc<P>,
        session_repository: Arc<E>,
        snapshot_repository: Arc<S>,
        redis_data_manager: Arc<R>,
    ) -> Self {
        Self {
            project_repository,
            session_repository,
            snapshot_repository,
            redis_data_manager,
        }
    }

    pub async fn get_project(
        &self,
        project_id: &str,
    ) -> Result<Option<Project>, ProjectServiceError> {
        Ok(self.project_repository.get_project(project_id).await?)
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
impl<P, E, S, R> ProjectRepository for ProjectService<P, E, S, R>
where
    P: ProjectRepository<Error = ProjectRepositoryError> + Send + Sync,
    E: ProjectEditingSessionRepository<Error = ProjectRepositoryError> + Send + Sync,
    S: ProjectSnapshotRepository<Error = ProjectRepositoryError> + Send + Sync,
    R: RedisDataManager<Error = ProjectRepositoryError> + Send + Sync,
{
    type Error = ProjectServiceError;

    async fn get_project(&self, project_id: &str) -> Result<Option<Project>, Self::Error> {
        Ok(self.project_repository.get_project(project_id).await?)
    }
}

#[async_trait]
impl<P, E, S, R> ProjectEditingSessionRepository for ProjectService<P, E, S, R>
where
    P: ProjectRepository<Error = ProjectRepositoryError> + Send + Sync,
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
impl<P, E, S, R> ProjectSnapshotRepository for ProjectService<P, E, S, R>
where
    P: ProjectRepository<Error = ProjectRepositoryError> + Send + Sync,
    E: ProjectEditingSessionRepository<Error = ProjectRepositoryError> + Send + Sync,
    S: ProjectSnapshotRepository<Error = ProjectRepositoryError> + Send + Sync,
    R: RedisDataManager<Error = ProjectRepositoryError> + Send + Sync,
{
    type Error = ProjectServiceError;

    async fn create_snapshot(&self, snapshot: ProjectSnapshot) -> Result<(), Self::Error> {
        Ok(self.snapshot_repository.create_snapshot(snapshot).await?)
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

    async fn update_snapshot_data(
        &self,
        project_id: &str,
        snapshot_data: SnapshotData,
    ) -> Result<(), Self::Error> {
        Ok(self
            .snapshot_repository
            .update_snapshot_data(project_id, snapshot_data)
            .await?)
    }
}

#[async_trait]
impl<P, E, S, R> RedisDataManager for ProjectService<P, E, S, R>
where
    P: ProjectRepository<Error = ProjectRepositoryError> + Send + Sync,
    E: ProjectEditingSessionRepository<Error = ProjectRepositoryError> + Send + Sync,
    S: ProjectSnapshotRepository<Error = ProjectRepositoryError> + Send + Sync,
    R: RedisDataManager<Error = ProjectRepositoryError> + Send + Sync,
{
    type Error = ProjectServiceError;

    async fn push_update(&self, update: Vec<u8>, updated_by: String) -> Result<(), Self::Error> {
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
