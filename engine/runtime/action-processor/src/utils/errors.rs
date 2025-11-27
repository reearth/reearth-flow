use thiserror::Error;

#[derive(Error, Debug)]
#[cfg_attr(feature = "analyzer", derive(reearth_flow_analyzer_core::DataSize))]
pub enum ProcessorUtilError {
    #[error("Decompressor error: {0}")]
    Decompressor(String),
}

pub type Result<T, E = ProcessorUtilError> = std::result::Result<T, E>;
