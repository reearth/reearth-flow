use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DocumentId(pub String);

impl DocumentId {
    pub fn new(id: String) -> Self {
        Self(id)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for DocumentId {
    fn from(id: String) -> Self {
        Self::new(id)
    }
}

impl From<&str> for DocumentId {
    fn from(id: &str) -> Self {
        Self::new(id.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct UpdateVersion(pub u64);

impl UpdateVersion {
    pub fn new(version: u64) -> Self {
        Self(version)
    }

    pub fn value(&self) -> u64 {
        self.0
    }

    pub fn increment(&self) -> Self {
        Self(self.0 + 1)
    }
}

impl From<u64> for UpdateVersion {
    fn from(version: u64) -> Self {
        Self::new(version)
    }
}

impl From<u32> for UpdateVersion {
    fn from(version: u32) -> Self {
        Self::new(version as u64)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StateVector(pub Vec<u8>);

impl StateVector {
    pub fn new(data: Vec<u8>) -> Self {
        Self(data)
    }

    pub fn empty() -> Self {
        Self(Vec::new())
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }
}

impl From<Vec<u8>> for StateVector {
    fn from(data: Vec<u8>) -> Self {
        Self::new(data)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentUpdate {
    pub data: Vec<u8>,
    pub version: UpdateVersion,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl DocumentUpdate {
    pub fn new(data: Vec<u8>, version: UpdateVersion) -> Self {
        Self {
            data,
            version,
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn with_timestamp(
        data: Vec<u8>,
        version: UpdateVersion,
        timestamp: chrono::DateTime<chrono::Utc>,
    ) -> Self {
        Self {
            data,
            version,
            timestamp,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(pub String);

impl UserId {
    pub fn new(id: String) -> Self {
        Self(id)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for UserId {
    fn from(id: String) -> Self {
        Self::new(id)
    }
}

impl From<&str> for UserId {
    fn from(id: &str) -> Self {
        Self::new(id.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(pub String);

impl SessionId {
    pub fn new(id: String) -> Self {
        Self(id)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn generate() -> Self {
        Self::new(uuid::Uuid::new_v4().to_string())
    }
}

impl From<String> for SessionId {
    fn from(id: String) -> Self {
        Self::new(id)
    }
}
