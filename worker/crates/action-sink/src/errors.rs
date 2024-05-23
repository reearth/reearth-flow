use thiserror::Error;

#[derive(Error, Debug)]
pub enum SinkError {
    #[error("Build factory error: {0}")]
    BuildFactory(String),
    #[error("File Writer error: {0}")]
    FileWriter(String),
}

impl SinkError {
    pub fn file_writer<T: ToString>(message: T) -> Self {
        Self::FileWriter(message.to_string())
    }
}
