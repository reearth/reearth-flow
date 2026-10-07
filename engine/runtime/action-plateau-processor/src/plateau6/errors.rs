use thiserror::Error;

/// Errors returned by the PLATEAU 6 generation-specific check actions.
#[allow(dead_code)]
#[derive(Error, Debug)]
pub(super) enum Plateau6ProcessorError {
    #[error("Unshared Edge Extractor Factory error: {0}")]
    UnsharedEdgeExtractorFactory(String),
}
