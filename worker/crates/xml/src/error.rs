#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Broken XML: {0}")]
    XmlError(#[from] quick_xml::Error),

    #[error("Not supported: {0}")]
    NotSupported(String),

    #[error("Syntax: {0}")]
    Syntax(String),

    #[error("InvalidCharacter: {0}")]
    InvalidCharacter(String),

    #[error("InvalidState")]
    InvalidState,

    #[error("IndexSize")]
    IndexSize,

    #[error("HierarchyRequest")]
    HierarchyRequest,

    #[error("NotFound")]
    NotFound,

    #[error("Namespace")]
    Namespace,

    #[error("WrongDocument")]
    WrongDocument,

    #[error("InvalidModification")]
    Malformed,
}

impl Eq for Error {}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::NotSupported(a), Self::NotSupported(b)) => a == b,
            (Self::Syntax(a), Self::Syntax(b)) => a == b,
            (Self::InvalidCharacter(a), Self::InvalidCharacter(b)) => a == b,
            _ => false,
        }
    }
}
