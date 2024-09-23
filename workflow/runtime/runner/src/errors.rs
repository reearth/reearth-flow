use std::io;

use reearth_flow_runtime::errors::ExecutionError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to find any build")]
    NoBuildFound,
    #[error("Missing api config or security input")]
    MissingSecurityConfig,
    #[error("Failed to server pgwire: {0}")]
    PGWireServerFailed(#[source] std::io::Error),
    #[error("Cache {0} has reached its maximum size. Try to increase `cache_max_map_size` in the config.")]
    CacheFull(String),
    #[error("Internal thread panic: {0}")]
    JoinError(#[source] tokio::task::JoinError),
    #[error("table_name: {0:?} not found in any of the connections")]
    SourceValidationError(String),
    #[error("connection: {0:?} not found")]
    ConnectionNotFound(String),
    #[error("Pipeline validation failed")]
    PipelineValidationError,
    #[error(transparent)]
    ExecutionError(#[from] ExecutionError),
    #[error("Output table {0} not used in any sink")]
    OutputTableNotUsed(String),
    #[error("Table name specified in sink not found: {0:?}")]
    SinkTableNotFound(String),
    #[error("No sinks initialized in the config provided")]
    EmptySinks,
    #[error("Failed to read organisation name. Error: {0}")]
    FailedToReadOrganisationName(#[source] io::Error),
    #[error("Command was aborted")]
    Aborted,
    #[error("This feature is only supported in enterprise: {0}")]
    UnsupportedFeature(String),
    #[error("Runtime Error: {0}")]
    RuntimeError(String),
}
