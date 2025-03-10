use thiserror::Error;

#[allow(dead_code)]
#[derive(Error, Debug)]
pub(super) enum PlateauProcessorError {
    #[error("UDXFolder Extractor Factory error: {0}")]
    UDXFolderExtractorFactory(String),
    #[error("UDXFolder Extractor error: {0}")]
    UDXFolderExtractor(String),
    #[error("MaxLod Extractor Factory error: {0}")]
    MaxLodExtractorFactory(String),
    #[error("MaxLod Extractor error: {0}")]
    MaxLodExtractor(String),
    #[error("AttributeFlattener error: {0}")]
    AttributeFlattener(String),
    #[error("CityCode Extractor Factory error: {0}")]
    CityCodeExtractorFactory(String),
    #[error("CityCode Extractor error: {0}")]
    CityCodeExtractor(String),
    #[error("ObjectList Extractor Factory error: {0}")]
    ObjectListExtractorFactory(String),
    #[error("ObjectList Extractor error: {0}")]
    ObjectListExtractor(String),
    #[error("MissingAttributeDetector Factory error: {0}")]
    MissingAttributeDetectorFactory(String),
    #[error("MissingAttributeDetector error: {0}")]
    MissingAttributeDetector(String),
    #[error("DomainOfDefinitionValidator error: {0}")]
    DomainOfDefinitionValidator(String),
}

pub(super) type Result<T, E = PlateauProcessorError> = std::result::Result<T, E>;
