use crate::error::ProjectServiceError;
use async_trait::async_trait;
use flow_websocket_domain::project::ProjectEditingSession;
use flow_websocket_domain::projection::{Action, Project, ProjectAllowedActions};
use flow_websocket_domain::repository::{
    ProjectEditingSessionRepository, ProjectRepository, ProjectSnapshotRepository,
};
use flow_websocket_domain::snapshot::ObjectTenant;
use flow_websocket_domain::utils::generate_id;
use flow_websocket_infra::persistence::project_repository::ProjectRepositoryError;
use std::sync::Arc;

pub struct ProjectService<P, E, S> {
    project_repository: Arc<P>,
    session_repository: Arc<E>,
    snapshot_repository: Arc<S>,
}

impl<P, E, S> ProjectService<P, E, S>
where
    P: ProjectRepository<Error = ProjectRepositoryError> + Send + Sync,
    E: ProjectEditingSessionRepository<Error = ProjectServiceError> + Send + Sync,
    S: ProjectSnapshotRepository<Error = ProjectRepositoryError> + Send + Sync,
{
    pub fn new(
        project_repository: Arc<P>,
        session_repository: Arc<E>,
        snapshot_repository: Arc<S>,
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
                // "REDIS_URL".to_owned(),
                ObjectTenant::new(generate_id(14, "tenant"), "tenant".to_owned()),
            ),
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
impl<P, E, S> ProjectRepository for ProjectService<P, E, S>
where
    P: ProjectRepository<Error = ProjectRepositoryError> + Send + Sync,
    E: ProjectEditingSessionRepository<Error = ProjectRepositoryError> + Send + Sync,
    S: ProjectSnapshotRepository<Error = ProjectRepositoryError> + Send + Sync,
{
    type Error = ProjectServiceError;

    async fn get_project(&self, project_id: &str) -> Result<Option<Project>, Self::Error> {
        Ok(self.project_repository.get_project(project_id).await?)
    }
}

#[async_trait]
impl<P, E, S> ProjectEditingSessionRepository for ProjectService<P, E, S>
where
    P: ProjectRepository<Error = ProjectRepositoryError> + Send + Sync,
    E: ProjectEditingSessionRepository<Error = ProjectRepositoryError> + Send + Sync,
    S: ProjectSnapshotRepository<Error = ProjectRepositoryError> + Send + Sync,
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
}
