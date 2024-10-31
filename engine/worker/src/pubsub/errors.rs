#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to encode : {0}")]
    Encode(String),
    #[error("Failed to publish message: {0}")]
    Publish(String),
}

impl Error {
    pub(crate) fn encode<T: ToString>(message: T) -> Self {
        Self::Encode(message.to_string())
    }

    pub(crate) fn publish<T: ToString>(message: T) -> Self {
        Self::Publish(message.to_string())
    }
}
