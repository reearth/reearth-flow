use std::fmt::{Debug, Display};

use bytes::Bytes;

use super::topic::Topic;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedMessage<M> {
    pub id: uuid::Uuid,
    /// The timestamp when message was created in the publishing service.
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// The message data.
    pub data: M,
}

impl ValidatedMessage<Bytes> {
    /// Create a new validated message
    pub fn new<D>(id: uuid::Uuid, timestamp: chrono::DateTime<chrono::Utc>, data: D) -> Self
    where
        D: Into<Bytes>,
    {
        Self {
            id,
            timestamp,
            data: data.into(),
        }
    }
}

pub trait EncodableMessage: Send + Sync + Debug {
    type Error: Send + Sync + Debug + Display;

    fn topic(&self) -> Topic;

    /// Encode the message payload.
    fn encode(&self) -> Result<ValidatedMessage<Bytes>, Self::Error>;
}
