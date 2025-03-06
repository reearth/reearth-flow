use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Init error: {0}")]
    Init(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Fetch error: {0}")]
    Fetch(String),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

impl Error {
    pub fn init<T: ToString>(message: T) -> Self {
        Self::Init(message.to_string())
    }

    pub fn fetch<T: ToString>(message: T) -> Self {
        Self::Fetch(message.to_string())
    }
}
