#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("ColorUtilError: {0}")]
    Color(String),

    #[error("CSVUtilError: {0}")]
    Csv(String),

    #[error("FSError: {0}")]
    Fs(String),

    #[error("SerdeUtilError: {0}")]
    Serde(String),

    #[error("URIUtilError: {0}")]
    Uri(String),

    #[error("XMLUtilError: {0}")]
    Xml(String),

    #[error("DirError: {0}")]
    Dir(String),

    #[error("StrError: {0}")]
    Str(String),

    #[error("JsonError: {0}")]
    Json(String),

    #[error("CompressError: {0}")]
    Compress(String),

    #[error("ZipError: {0}")]
    Zip(String),

    #[error("DatetimeError: {0}")]
    Datetime(String),
}

impl Error {
    pub fn color<T: ToString>(message: T) -> Self {
        Self::Color(message.to_string())
    }

    pub fn csv<T: ToString>(message: T) -> Self {
        Self::Csv(message.to_string())
    }

    pub fn fs<T: ToString>(message: T) -> Self {
        Self::Fs(message.to_string())
    }

    pub fn serde<T: ToString>(message: T) -> Self {
        Self::Serde(message.to_string())
    }

    pub fn uri<T: ToString>(message: T) -> Self {
        Self::Uri(message.to_string())
    }

    pub fn xml<T: ToString>(message: T) -> Self {
        Self::Xml(message.to_string())
    }

    pub fn dir<T: ToString>(message: T) -> Self {
        Self::Dir(message.to_string())
    }

    pub fn json<T: ToString>(message: T) -> Self {
        Self::Json(message.to_string())
    }

    pub fn compress<T: ToString>(message: T) -> Self {
        Self::Compress(message.to_string())
    }

    pub fn zip<T: ToString>(message: T) -> Self {
        Self::Zip(message.to_string())
    }

    pub fn datetime<T: ToString>(message: T) -> Self {
        Self::Datetime(message.to_string())
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub mod collection;
pub mod color;
pub mod compress;
pub mod csv;
pub mod datetime;
pub mod dir;
pub mod fs;
pub mod future;
pub mod image;
pub mod json;
pub mod runtime_config;
pub mod serde;
pub mod str;
pub mod texture;
pub mod uri;
pub mod xml;
pub mod zip;
