use thiserror::Error;

/// Errors returned by the PLATEAU 6 generation-specific check actions.
#[allow(dead_code)]
#[derive(Error, Debug)]
pub(super) enum Plateau6ProcessorError {
    #[error("Unshared Edge Extractor Factory error: {0}")]
    UnsharedEdgeExtractorFactory(String),
    #[error("Flooding Area Surface Generator Factory error: {0}")]
    FloodingAreaSurfaceGeneratorFactory(String),
    #[error("Flooding Area Surface Generator error: {0}")]
    FloodingAreaSurfaceGenerator(String),
}
