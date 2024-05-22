use thiserror::Error;

#[derive(Error, Debug)]
pub enum UniversalSourceError {
    #[error("Build factory error: {0}")]
    BuildFactory(String),
    #[error("File Reader error: {0}")]
    FileReader(String),
    #[error("File Path Extractor error: {0}")]
    FilePathExtractor(String),
}

pub type Result<T, E = UniversalSourceError> = std::result::Result<T, E>;
