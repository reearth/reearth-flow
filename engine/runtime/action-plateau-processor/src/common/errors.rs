use thiserror::Error;

/// Errors returned by the PLATEAU generation-independent common check actions.
#[allow(dead_code)]
#[derive(Error, Debug)]
pub(crate) enum PlateauProcessorError {
    #[error("UDXFolder Extractor Factory error: {0}")]
    UDXFolderExtractorFactory(String),
    #[error("UDXFolder Extractor error: {0}")]
    UDXFolderExtractor(String),
    #[error("DomainOfDefinitionValidator Factory error: {0}")]
    DomainOfDefinitionValidatorFactory(String),
    #[error("DomainOfDefinitionValidator error: {0}")]
    DomainOfDefinitionValidator(String),
    #[error("ObjectListExtractor Factory error: {0}")]
    ObjectListExtractorFactory(String),
    #[error("ObjectListExtractor error: {0}")]
    ObjectListExtractor(String),
    #[error("MissingAttributeDetector Factory error: {0}")]
    MissingAttributeDetectorFactory(String),
    #[error("MissingAttributeDetector error: {0}")]
    MissingAttributeDetector(String),
}

pub(crate) type Result<T, E = PlateauProcessorError> = std::result::Result<T, E>;
