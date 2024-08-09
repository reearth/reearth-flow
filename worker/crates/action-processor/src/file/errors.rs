use thiserror::Error;

#[derive(Error, Debug)]
pub enum FileProcessorError {
    #[error("Property Extractor Factory error: {0}")]
    PropertyExtractorFactory(String),
    #[error("Property Extractor error: {0}")]
    PropertyExtractor(String),
}

pub type Result<T, E = FileProcessorError> = std::result::Result<T, E>;
