use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct HistoryItem {
    pub version: u64,
    pub updates: Vec<u8>,
    pub timestamp: chrono::DateTime<Utc>,
}

impl HistoryItem {
    pub fn new(version: u64, updates: Vec<u8>, timestamp: chrono::DateTime<Utc>) -> Self {
        Self {
            version,
            updates,
            timestamp,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct DocumentResponse {
    pub id: String,
    pub updates: Vec<u8>,
    pub version: u64,
    pub timestamp: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetLatestRequest {
    pub doc_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetHistoryRequest {
    pub doc_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct RollbackRequest {
    pub doc_id: String,
    pub version: u64,
}

#[derive(Serialize, Deserialize)]
pub struct CreateSnapshotRequest {
    pub doc_id: String,
    pub version: u64,
    pub name: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct SnapshotResponse {
    pub id: String,
    pub updates: Vec<u8>,
    pub version: u64,
    pub timestamp: String,
    pub name: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct HistoryResponse {
    pub updates: Vec<u8>,
    pub version: u64,
    pub timestamp: String,
}

#[derive(serde::Serialize)]
pub struct HistoryMetadataResponse {
    pub version: u64,
    pub timestamp: String,
}

#[derive(Serialize, Deserialize)]
pub struct ImportDocumentRequest {
    pub data: Vec<u8>,
}
