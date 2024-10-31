use chrono::OutOfRangeError;
use flow_websocket_domain::editing_session::ProjectEditingSessionError;
use flow_websocket_infra::persistence::{
    project_repository::ProjectRepositoryError, redis::errors::FlowProjectRedisDataManagerError,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProjectServiceError {
    #[error(transparent)]
    Repository(#[from] ProjectRepositoryError),

    #[error(transparent)]
    EditingSession(#[from] ProjectEditingSessionError),

    #[error(transparent)]
    FlowProjectRedisDataManager(#[from] FlowProjectRedisDataManagerError),

    #[error("Session not setup")]
    SessionNotSetup,

    #[error(transparent)]
    ChronoDurationConversionError(OutOfRangeError),
}
