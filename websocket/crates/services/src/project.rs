use async_trait::async_trait;
use flow_websocket_domain::project::{Project, ProjectAllowedActions, ProjectEditingSession};
use flow_websocket_domain::repository::{ProjectRepository, ProjectEditingSessionRepository, ProjectSnapshotRepository};
use std::sync::Arc;
use std::error::Error;

pub struct ProjectService {
    project_repository: Arc<dyn ProjectRepository>,
    session_repository: Arc<dyn ProjectEditingSessionRepository>,
    snapshot_repository: Arc<dyn ProjectSnapshotRepository>,
}

impl ProjectService {
    pub fn new(
        project_repository: Arc<dyn ProjectRepository>,
        session_repository: Arc<dyn ProjectEditingSessionRepository>,
        snapshot_repository: Arc<dyn ProjectSnapshotRepository>,
    ) -> Self {
        Self {
            project_repository,
            session_repository,
            snapshot_repository,
        }
    }

    pub async fn get_project(&self, project_id: &str) -> Result<Option<Project>, Box<dyn Error>> {
        self.project_repository.get_project(project_id).await
    }

    pub async fn get_or_create_editing_session(&self, project_id: &str) -> Result<ProjectEditingSession, Box<dyn Error>> {
        let mut session = match self.session_repository.get_active_session(project_id).await? {
            Some(session) => session,
            None => ProjectEditingSession::new(project_id.to_string(), "REDIS_URL".to_owned()),
        };

        if session.session_id.is_none() {
            session.start_or_join_session(&*self.snapshot_repository).await?;
            self.session_repository.create_session(session.clone()).await?;
        }

        Ok(session)
    }

    pub async fn get_project_allowed_actions(&self, project_id: &str, actions: Vec<String>) -> Result<ProjectAllowedActions, Box<dyn Error>> {
        // This is a placeholder implementation. You should implement the actual logic
        // to determine allowed actions based on certain requirements we'll have decide @pyshx
        Ok(ProjectAllowedActions {
            id: project_id.to_string(),
            actions: actions.into_iter().map(|action| flow_websocket_domain::project::Action {
                action,
                allowed: true,
            }).collect(),
        })
    }
}

#[async_trait]
impl ProjectRepository for ProjectService {
    async fn get_project(&self, project_id: &str) -> Result<Option<Project>, Box<dyn Error>> {
        self.project_repository.get_project(project_id).await
    }
}

#[async_trait]
impl ProjectEditingSessionRepository for ProjectService {
    async fn create_session(&self, session: ProjectEditingSession) -> Result<(), Box<dyn Error>> {
        self.session_repository.create_session(session).await
    }

    async fn get_active_session(&self, project_id: &str) -> Result<Option<ProjectEditingSession>, Box<dyn Error>> {
        self.session_repository.get_active_session(project_id).await
    }

    async fn update_session(&self, session: ProjectEditingSession) -> Result<(), Box<dyn Error>> {
        self.session_repository.update_session(session).await
    }
}
