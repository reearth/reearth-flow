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
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub mod collection;
pub mod color;
pub mod csv;
pub mod fs;
pub mod serde;
pub mod str;
pub mod uri;
pub mod xml;
