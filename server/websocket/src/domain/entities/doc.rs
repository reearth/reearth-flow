use chrono::{DateTime, Utc};

use crate::shared::errors::AppError;
use crate::shared::result::AppResult;
use crate::shared::utils::ensure_not_empty;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DocId(String);

impl DocId {
    pub fn new(value: impl Into<String>) -> AppResult<Self> {
        let value = value.into();
        ensure_not_empty(&value, "doc_id")?;
        Ok(Self(value))
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for DocId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct DocumentVersion(u64);

impl DocumentVersion {
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

#[derive(Debug, Clone)]
pub struct Document {
    pub id: DocId,
    pub updates: Vec<u8>,
    pub version: DocumentVersion,
    pub timestamp: DateTime<Utc>,
}

impl Document {
    pub fn new(
        id: DocId,
        updates: Vec<u8>,
        version: DocumentVersion,
        timestamp: DateTime<Utc>,
    ) -> AppResult<Self> {
        if updates.is_empty() {
            return Err(AppError::invalid_input("document updates cannot be empty"));
        }
        Ok(Self {
            id,
            updates,
            version,
            timestamp,
        })
    }

    pub fn from_raw(
        id: impl Into<String>,
        updates: Vec<u8>,
        version: u64,
        timestamp: DateTime<Utc>,
    ) -> AppResult<Self> {
        let doc_id = DocId::new(id)?;
        let doc_version = DocumentVersion::new(version);
        Self::new(doc_id, updates, doc_version, timestamp)
    }
}

#[derive(Debug, Clone)]
pub struct HistoryItem {
    pub version: DocumentVersion,
    pub updates: Vec<u8>,
    pub timestamp: DateTime<Utc>,
}

impl HistoryItem {
    pub fn new(version: DocumentVersion, updates: Vec<u8>, timestamp: DateTime<Utc>) -> Self {
        Self {
            version,
            updates,
            timestamp,
        }
    }

    pub fn from_raw(version: u64, updates: Vec<u8>, timestamp: DateTime<Utc>) -> Self {
        Self::new(DocumentVersion::new(version), updates, timestamp)
    }
}
