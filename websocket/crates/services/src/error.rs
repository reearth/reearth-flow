use flow_websocket_domain::project::ProjectEditingSessionError;
use flow_websocket_infra::persistence::project_repository::ProjectRepositoryError;
use std::fmt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProjectServiceError {
    #[error(transparent)]
    RepositoryError(#[from] ProjectRepositoryError),

    #[error("Session not setup")]
    SessionNotSetup,

    #[error("Snapshot repository error: {0}")]
    SnapshotRepositoryError(String),

    #[error("Unexpected error: {0}")]
    UnexpectedError(String),
}

impl<E> From<ProjectEditingSessionError<E>> for ProjectServiceError
where
    E: fmt::Debug + fmt::Display,
{
    fn from(err: ProjectEditingSessionError<E>) -> Self {
        match err {
            ProjectEditingSessionError::SessionNotSetup => ProjectServiceError::SessionNotSetup,
            ProjectEditingSessionError::SnapshotRepository(repo_err) => {
                ProjectServiceError::SnapshotRepositoryError(format!("{}", repo_err))
            }
        }
    }
}
