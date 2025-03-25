use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct Document {
    pub id: String,
    pub updates: Vec<u8>,
    pub version: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct HistoryItem {
    pub version: u64,
    pub updates: Vec<u8>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
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
