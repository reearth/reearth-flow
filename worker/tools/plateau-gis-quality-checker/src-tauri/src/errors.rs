use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum Error {
    #[error("I/O error: {0}")]
    Io(String),
    #[error("Invalid path: {0}")]
    InvalidPath(String),
    #[error("Setup Error: {0}")]
    Setup(String),
    #[error("Execute failed: {0}")]
    ExecuteFailed(String),
}

impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err.to_string())
    }
}

impl Error {
    pub(crate) fn invalid_path<T: ToString>(message: T) -> Self {
        Self::InvalidPath(message.to_string())
    }

    pub(crate) fn io<T: ToString>(message: T) -> Self {
        Self::Io(message.to_string())
    }

    pub(crate) fn setup<T: ToString>(message: T) -> Self {
        Self::Setup(message.to_string())
    }

    pub(crate) fn execute_failed<T: ToString>(message: T) -> Self {
        Self::ExecuteFailed(message.to_string())
    }
}
