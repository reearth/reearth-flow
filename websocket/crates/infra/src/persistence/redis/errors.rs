use thiserror::Error;

use super::flow_project_lock::LockError;

#[derive(Error, Debug)]
pub enum FlowProjectRedisDataManagerError {
    #[error(transparent)]
    Redis(#[from] redis::RedisError),
    #[error("Serde JSON error: {0}")]
    SerdeJson(#[from] serde_json::Error),
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
    #[error("Pool run error: {0}")]
    PoolRunError(#[from] bb8::RunError<redis::RedisError>),
    #[error(transparent)]
    LockError(#[from] LockError),
    #[error(transparent)]
    Yrs(#[from] yrs::error::UpdateError),
}
