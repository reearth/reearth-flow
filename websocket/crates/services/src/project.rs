use crate::error::ProjectServiceError;
use async_trait::async_trait;
use flow_websocket_domain::project::ProjectEditingSession;
use flow_websocket_domain::projection::{Action, Project, ProjectAllowedActions};
use flow_websocket_domain::repository::{
    ProjectEditingSessionRepository, ProjectRepository, ProjectSnapshotRepository,
};
use flow_websocket_infra::persistence::project_repository::ProjectRepositoryError;
use std::sync::Arc;

pub struct ProjectService {
    project_repository: Arc<dyn ProjectRepository<Error = ProjectRepositoryError> + Send + Sync>,
    session_repository:
        Arc<dyn ProjectEditingSessionRepository<Error = ProjectRepositoryError> + Send + Sync>,
    snapshot_repository:
        Arc<dyn ProjectSnapshotRepository<Error = ProjectRepositoryError> + Send + Sync>,
}

impl ProjectService {
    pub fn new(
        project_repository: Arc<
            dyn ProjectRepository<Error = ProjectRepositoryError> + Send + Sync,
        >,
        session_repository: Arc<
            dyn ProjectEditingSessionRepository<Error = ProjectRepositoryError> + Send + Sync,
        >,
        snapshot_repository: Arc<
            dyn ProjectSnapshotRepository<Error = ProjectRepositoryError> + Send + Sync,
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
    ) -> Result<Option<Project>, ProjectServiceError> {
        self.project_repository
            .get_project(project_id)
            .await
            .map_err(ProjectServiceError::RepositoryError)
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
            None => ProjectEditingSession::new(project_id.to_string(), "REDIS_URL".to_owned()),
        };

        if session.session_id.is_none() {
            session
                .start_or_join_session(&*self.snapshot_repository)
                .await
                .map_err(ProjectServiceError::EditingSessionError)?;
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
impl ProjectRepository for ProjectService {
    type Error = ProjectServiceError;

    async fn get_project(&self, project_id: &str) -> Result<Option<Project>, Self::Error> {
        self.project_repository
            .get_project(project_id)
            .await
            .map_err(ProjectServiceError::RepositoryError)
    }
}

#[async_trait]
impl ProjectEditingSessionRepository for ProjectService {
    type Error = ProjectServiceError;

    async fn create_session(&self, session: ProjectEditingSession) -> Result<(), Self::Error> {
        self.session_repository
            .create_session(session)
            .await
            .map_err(ProjectServiceError::RepositoryError)
    }

    async fn get_active_session(
        &self,
        project_id: &str,
    ) -> Result<Option<ProjectEditingSession>, Self::Error> {
        self.session_repository
            .get_active_session(project_id)
            .await
            .map_err(ProjectServiceError::RepositoryError)
    }

    async fn update_session(&self, session: ProjectEditingSession) -> Result<(), Self::Error> {
        self.session_repository
            .update_session(session)
            .await
            .map_err(ProjectServiceError::RepositoryError)
    }
}
