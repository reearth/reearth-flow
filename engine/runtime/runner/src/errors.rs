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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_no_build_found() {
        let error = Error::NoBuildFound;
        assert_eq!(error.to_string(), "Failed to find any build");
    }

    #[test]
    fn test_error_missing_security_config() {
        let error = Error::MissingSecurityConfig;
        assert_eq!(error.to_string(), "Missing api config or security input");
    }

    #[test]
    fn test_error_cache_full() {
        let error = Error::CacheFull("test_cache".to_string());
        assert!(error.to_string().contains("test_cache"));
        assert!(error.to_string().contains("maximum size"));
    }

    #[test]
    fn test_error_connection_not_found() {
        let error = Error::ConnectionNotFound("db_connection".to_string());
        assert!(error.to_string().contains("db_connection"));
    }

    #[test]
    fn test_error_pipeline_validation() {
        let error = Error::PipelineValidationError;
        assert_eq!(error.to_string(), "Pipeline validation failed");
    }

    #[test]
    fn test_error_output_table_not_used() {
        let error = Error::OutputTableNotUsed("plateau_buildings".to_string());
        assert!(error.to_string().contains("plateau_buildings"));
    }

    #[test]
    fn test_error_sink_table_not_found() {
        let error = Error::SinkTableNotFound("output_table".to_string());
        assert!(error.to_string().contains("output_table"));
    }

    #[test]
    fn test_error_empty_sinks() {
        let error = Error::EmptySinks;
        assert!(error.to_string().contains("No sinks"));
    }

    #[test]
    fn test_error_aborted() {
        let error = Error::Aborted;
        assert_eq!(error.to_string(), "Command was aborted");
    }

    #[test]
    fn test_error_unsupported_feature() {
        let error = Error::UnsupportedFeature("advanced_analytics".to_string());
        assert!(error.to_string().contains("enterprise"));
        assert!(error.to_string().contains("advanced_analytics"));
    }

    #[test]
    fn test_error_runtime_error() {
        let error = Error::RuntimeError("Unexpected failure".to_string());
        assert!(error.to_string().contains("Unexpected failure"));
    }
}

