use thiserror::Error;

#[derive(Error, Debug)]
pub enum FlowProjectRedisDataManagerError {
    #[error(transparent)]
    Redis(#[from] redis::RedisError),
    #[error("Serde JSON error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("Another Editing Session in progress")]
    EditingSessionInProgress,
    #[error("Failed to merge updates")]
    MergeUpdates,
    #[error("Failed to get last update id")]
    LastUpdateId,
    #[error("Missing state update - Key: {key}, Context: {context}")]
    MissingStateUpdate { key: String, context: String },
    #[error("Session not set for project {project_id}")]
    SessionNotSet { project_id: String },
    #[error(transparent)]
    DecodeUpdate(#[from] yrs::encoding::read::Error),
    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),
    #[error(transparent)]
    Join(#[from] tokio::task::JoinError),
    #[error("Unknown error: {0}")]
    Unknown(String),
    #[error("Failed to acquire lock")]
    LockError,
    #[error("Pool run error: {0}")]
    PoolRunError(#[from] bb8::RunError<redis::RedisError>),
    #[error("Global Lock Error: {0:?}")]
    FlowProjectLock(rslock::LockError),
}

impl From<rslock::LockError> for FlowProjectRedisDataManagerError {
    fn from(err: rslock::LockError) -> Self {
        FlowProjectRedisDataManagerError::FlowProjectLock(err)
    }
}
