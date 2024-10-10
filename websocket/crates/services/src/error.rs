use flow_websocket_domain::project::ProjectEditingSessionError;
use flow_websocket_infra::persistence::project_repository::ProjectRepositoryError;
use std::fmt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProjectServiceError {
    #[error(transparent)]
    Repository(#[from] ProjectRepositoryError),

    #[error("Session not setup")]
    SessionNotSetup,

    #[error("Snapshot repository error: {0}")]
    SnapshotRepository(String),

    #[error("Unexpected error: {0}")]
    Unexpected(String),
}

impl<S, R> From<ProjectEditingSessionError<S, R>> for ProjectServiceError
where
    S: fmt::Debug,
    R: fmt::Debug,
{
    fn from(err: ProjectEditingSessionError<S, R>) -> Self {
        match err {
            ProjectEditingSessionError::SessionNotSetup => ProjectServiceError::SessionNotSetup,
            ProjectEditingSessionError::Snapshot(repo_err) => {
                ProjectServiceError::SnapshotRepository(format!("{:?}", repo_err))
            }
            ProjectEditingSessionError::Redis(repo_err) => {
                ProjectServiceError::SnapshotRepository(format!("{:?}", repo_err))
            }
            ProjectEditingSessionError::Custom(err) => ProjectServiceError::Unexpected(err),
        }
    }
}
