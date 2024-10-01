use flow_websocket_domain::project::ProjectEditingSessionError;
use flow_websocket_infra::persistence::project_repository::ProjectRepositoryError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProjectServiceError {
    #[error(transparent)]
    RepositoryError(#[from] ProjectRepositoryError),

    #[error(transparent)]
    EditingSessionError(#[from] ProjectEditingSessionError<ProjectRepositoryError>),
}
