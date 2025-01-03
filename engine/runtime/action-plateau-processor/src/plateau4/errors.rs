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
}

pub(super) type Result<T, E = PlateauProcessorError> = std::result::Result<T, E>;
