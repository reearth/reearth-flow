use chrono::{DateTime, Utc};

use crate::domain::value_objects::doc_id::DocId;
use crate::domain::value_objects::document_version::DocumentVersion;
use crate::shared::errors::AppError;
use crate::shared::result::AppResult;

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
