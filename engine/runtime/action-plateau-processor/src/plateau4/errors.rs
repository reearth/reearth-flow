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
    #[error("Unmatched Xlink Detector Factory error: {0}")]
    UnmatchedXlinkDetectorFactory(String),
    #[error("Unmatched Xlink Detector error: {0}")]
    UnmatchedXlinkDetector(String),
    #[error("BuildingInstallationGeometryTypeChecker error: {0}")]
    BuildingInstallationGeometryTypeChecker(String),
    #[error("BuildingUsageAttributeValidator error: {0}")]
    BuildingUsageAttributeValidator(String),
    #[error("BuildingUsageAttributeValidator Factory error: {0}")]
    BuildingUsageAttributeValidatorFactory(String),
    #[error("BuildingPartConnectivityChecker error: {0}")]
    BuildingPartConnectivityChecker(String),
    #[error("BuildingPartConnectivityChecker Factory error: {0}")]
    BuildingPartConnectivityCheckerFactory(String),
    #[error("SolidIntersectionTestPairCreator error: {0}")]
    SolidIntersectionTestPairCreator(String),
    #[error("SolidIntersectionTestPairCreator Factory error: {0}")]
    SolidIntersectionTestPairCreatorFactory(String),
    #[error("TranXlinkDetector error: {0}")]
    TranXlinkDetector(String),
    #[error("TranXlinkDetector Factory error: {0}")]
    TranXlinkDetectorFactory(String),
}

pub(super) type Result<T, E = PlateauProcessorError> = std::result::Result<T, E>;
