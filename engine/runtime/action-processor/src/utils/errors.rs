use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProcessorUtilError {
    #[error("Decompressor error: {0}")]
    Decompressor(String),
}

pub type Result<T, E = ProcessorUtilError> = std::result::Result<T, E>;
