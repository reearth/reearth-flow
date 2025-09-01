use std::fmt;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InstanceId(String);

impl InstanceId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    pub fn from_string(id: String) -> Result<Self, InstanceIdError> {
        if id.is_empty() {
            return Err(InstanceIdError::Empty);
        }
        Ok(Self(id))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for InstanceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for InstanceId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Error)]
pub enum InstanceIdError {
    #[error("Instance ID cannot be empty")]
    Empty,
}
