use flow_websocket_domain::project::ProjectEditingSessionError;
use flow_websocket_infra::persistence::project_repository::ProjectRepositoryError;
use std::fmt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProjectServiceError {
    #[error(transparent)]
    RepositoryError(#[from] ProjectRepositoryError),

    #[error(transparent)]
    ProjectEditingSessionError(#[from] Box<ProjectEditingSessionError<ProjectServiceError>>),

    #[error("Unexpected error: {0}")]
    UnexpectedError(String),
}

impl From<ProjectEditingSessionError<ProjectRepositoryError>> for ProjectServiceError {
    fn from(err: ProjectEditingSessionError<ProjectRepositoryError>) -> Self {
        match err {
            ProjectEditingSessionError::SessionNotSetup => {
                ProjectServiceError::ProjectEditingSessionError(Box::new(
                    ProjectEditingSessionError::SessionNotSetup,
                ))
            }
            ProjectEditingSessionError::SnapshotRepository(repo_err) => {
                ProjectServiceError::RepositoryError(repo_err)
            }
        }
    }
}

impl From<ProjectEditingSessionError<ProjectServiceError>> for ProjectServiceError {
    fn from(err: ProjectEditingSessionError<ProjectServiceError>) -> Self {
        ProjectServiceError::ProjectEditingSessionError(Box::new(err))
    }
}
