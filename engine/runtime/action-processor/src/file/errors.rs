use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum FileProcessorError {
    #[error("Property Extractor Factory error: {0}")]
    PropertyExtractorFactory(String),
    #[error("Property Extractor error: {0}")]
    PropertyExtractor(String),
    #[error("Directory Decompressor Factory error: {0}")]
    DirectoryDecompressorFactory(String),
    #[error("Directory Decompressor error: {0}")]
    DirectoryDecompressor(String),
}

pub(crate) type Result<T, E = FileProcessorError> = std::result::Result<T, E>;
