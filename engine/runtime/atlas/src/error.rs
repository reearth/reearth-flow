use thiserror::Error;

#[derive(Error, Debug)]
pub enum AtlasError {
    #[error("Atlas builder error: {0}")]
    Builder(String),
    #[error("Failed to open texture '{}': {source}", path.display())]
    TextureLoad {
        path: std::path::PathBuf,
        source: image::ImageError,
    },
}

impl AtlasError {
    pub fn builder<T: ToString>(message: T) -> Self {
        Self::Builder(message.to_string())
    }
}

pub type Result<T, E = AtlasError> = std::result::Result<T, E>;
