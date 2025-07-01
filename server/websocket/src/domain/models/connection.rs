use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ConnectionId(String);

impl ConnectionId {
    pub fn new() -> Self {
        ConnectionId(Uuid::new_v4().to_string())
    }

    pub fn from_string(id: String) -> Self {
        ConnectionId(id)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for ConnectionId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub id: ConnectionId,
    pub document_id: String,
    pub instance_id: String,
}
