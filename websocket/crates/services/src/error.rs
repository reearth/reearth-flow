use chrono::OutOfRangeError;
use flow_websocket_domain::project::ProjectEditingSessionError;
use flow_websocket_infra::persistence::{
    project_repository::ProjectRepositoryError,
    redis::flow_project_redis_data_manager::FlowProjectRedisDataManagerError,
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
