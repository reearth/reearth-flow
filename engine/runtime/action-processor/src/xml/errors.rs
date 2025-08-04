use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub(crate) enum XmlProcessorError {
    #[error("Xml Fragmenter Factory error: {0}")]
    FragmenterFactory(String),
    #[error("Xml Fragmenter error: {0}")]
    Fragmenter(String),
    #[error("Xml Validator Factory error: {0}")]
    ValidatorFactory(String),
    #[error("Xml Validator error: {0}")]
    Validator(String),
}

pub(crate) type Result<T, E = XmlProcessorError> = std::result::Result<T, E>;
