use std::fmt;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConnectionId(String);

impl ConnectionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    pub fn from_string(id: String) -> Result<Self, ConnectionIdError> {
        if id.is_empty() {
            return Err(ConnectionIdError::Empty);
        }
        Ok(Self(id))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ConnectionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for ConnectionId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Error)]
pub enum ConnectionIdError {
    #[error("Connection ID cannot be empty")]
    Empty,
}
