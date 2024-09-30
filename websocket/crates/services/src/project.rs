use async_trait::async_trait;
use flow_websocket_domain::project::{Project, ProjectAllowedActions, ProjectEditingSession};
use flow_websocket_domain::repository::{
    ProjectEditingSessionRepository, ProjectRepository, ProjectSnapshotRepository,
};
use flow_websocket_infra::persistence::project_repository::ProjectRepositoryError;
use std::sync::Arc;

pub struct ProjectService {
    project_repository: Arc<dyn ProjectRepository<ProjectRepositoryError> + Send + Sync>,
    session_repository:
        Arc<dyn ProjectEditingSessionRepository<ProjectRepositoryError> + Send + Sync>,
    snapshot_repository: Arc<dyn ProjectSnapshotRepository<ProjectRepositoryError> + Send + Sync>,
}

impl ProjectService {
    pub fn new(
        project_repository: Arc<dyn ProjectRepository<ProjectRepositoryError> + Send + Sync>,
        session_repository: Arc<
            dyn ProjectEditingSessionRepository<ProjectRepositoryError> + Send + Sync,
        >,
        snapshot_repository: Arc<
            dyn ProjectSnapshotRepository<ProjectRepositoryError> + Send + Sync,
        >,
    ) -> Self {
        Self {
            project_repository,
            session_repository,
            snapshot_repository,
        }
    }

    pub async fn get_project(
        &self,
        project_id: &str,
    ) -> Result<Option<Project>, ProjectRepositoryError> {
        self.project_repository.get_project(project_id).await
    }

    pub async fn get_or_create_editing_session(
        &self,
        project_id: &str,
    ) -> Result<ProjectEditingSession, ProjectRepositoryError> {
        let mut session = match self
            .session_repository
            .get_active_session(project_id)
            .await?
        {
            Some(session) => session,
            None => ProjectEditingSession::new(project_id.to_string(), "REDIS_URL".to_owned()),
        };

        if session.session_id.is_none() {
            session
                .start_or_join_session(&*self.snapshot_repository)
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
    ) -> Result<ProjectAllowedActions, ProjectRepositoryError> {
        // This is a placeholder implementation. You should implement the actual logic
        // to determine allowed actions based on certain requirements we'll have decide @pyshx
        Ok(ProjectAllowedActions {
            id: project_id.to_string(),
            actions: actions
                .into_iter()
                .map(|action| flow_websocket_domain::project::Action {
                    action,
                    allowed: true,
                })
                .collect(),
        })
    }
}

#[async_trait]
impl ProjectRepository<ProjectRepositoryError> for ProjectService
where
    Self: Send + Sync,
{
    async fn get_project(
        &self,
        project_id: &str,
    ) -> Result<Option<Project>, ProjectRepositoryError> {
        self.project_repository.get_project(project_id).await
    }
}

#[async_trait]
impl ProjectEditingSessionRepository<ProjectRepositoryError> for ProjectService {
    async fn create_session(
        &self,
        session: ProjectEditingSession,
    ) -> Result<(), ProjectRepositoryError> {
        self.session_repository.create_session(session).await
    }

    async fn get_active_session(
        &self,
        project_id: &str,
    ) -> Result<Option<ProjectEditingSession>, ProjectRepositoryError> {
        self.session_repository.get_active_session(project_id).await
    }

    async fn update_session(
        &self,
        session: ProjectEditingSession,
    ) -> Result<(), ProjectRepositoryError> {
        self.session_repository.update_session(session).await
    }
}
