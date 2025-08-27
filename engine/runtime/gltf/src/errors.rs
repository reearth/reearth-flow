use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Gltf Metadata error: {0}")]
    Metadata(String),
    #[error("Gltf writer error: {0}")]
    Writer(String),
    #[error("Gltf compression error: {0}")]
    Compression(#[from] draco_oxide::io::gltf::transcoder::Err),
    #[error("std io error: {0}")]
    StdIo(#[from] std::io::Error),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

impl Error {
    pub fn metadata<T: ToString>(message: T) -> Self {
        Self::Metadata(message.to_string())
    }

    pub fn writer<T: ToString>(message: T) -> Self {
        Self::Writer(message.to_string())
    }
}
