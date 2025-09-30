use serde::{Deserialize, Serialize};

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

#[derive(Serialize)]
pub struct HistoryMetadataResponse {
    pub version: u64,
    pub timestamp: String,
}

#[derive(Serialize, Deserialize)]
pub struct ImportDocumentRequest {
    pub data: Vec<u8>,
}
