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
    #[error("BuildingUsageAttributeValidator Factory error: {0}")]
    BuildingUsageAttributeValidatorFactory(String),
    #[error("BuildingUsageAttributeValidator error: {0}")]
    BuildingUsageAttributeValidator(String),
    #[error("BuildingPartConnectivityChecker Factory error: {0}")]
    BuildingPartConnectivityCheckerFactory(String),
    #[error("BuildingPartConnectivityChecker error: {0}")]
    BuildingPartConnectivityChecker(String),
    #[error("SolidIntersectionTestPairCreator Factory error: {0}")]
    SolidIntersectionTestPairCreatorFactory(String),
    #[error("SolidIntersectionTestPairCreator error: {0}")]
    SolidIntersectionTestPairCreator(String),
    #[error("Unmatched Xlink Detector Factory error: {0}")]
    UnmatchedXlinkDetectorFactory(String),
    #[error("Unmatched Xlink Detector error: {0}")]
    UnmatchedXlinkDetector(String),
    #[error("DestinationMeshCodeExtractor Factory error: {0}")]
    DestinationMeshCodeExtractorFactory(String),
    #[error("DestinationMeshCodeExtractor error: {0}")]
    DestinationMeshCodeExtractor(String),
}

pub(crate) type Result<T, E = PlateauProcessorError> = std::result::Result<T, E>;
