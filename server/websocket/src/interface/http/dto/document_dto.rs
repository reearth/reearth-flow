use crate::infrastructure::storage::gcs::UpdateInfo;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use yrs::updates::encoder::Encode;

/// Request DTOs
#[derive(Serialize, Deserialize)]
pub struct CreateSnapshotRequest {
    pub doc_id: String,
    pub version: u64,
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
    pub version: u64,
}

/// Response DTOs
#[derive(Serialize, Deserialize)]
pub struct DocumentResponse {
    pub id: String,
    pub updates: Vec<u8>,
    pub version: u64,
    pub timestamp: String,
}

#[derive(Serialize, Deserialize)]
pub struct SnapshotResponse {
    pub success: bool,
    pub doc_id: String,
    pub version: u64,
    pub timestamp: String,
}

#[derive(Serialize, Deserialize)]
pub struct HistoryResponse {
    pub doc_id: String,
    pub history: Vec<HistoryItemDto>,
}

#[derive(serde::Serialize)]
pub struct HistoryMetadataResponse {
    pub versions: Vec<u64>,
}

/// Internal DTOs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentDto {
    pub id: String,
    pub updates: Vec<u8>,
    pub version: u64,
    pub timestamp: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryItemDto {
    pub version: u64,
    pub updates: Vec<u8>,
    pub timestamp: chrono::DateTime<Utc>,
}

impl From<UpdateInfo> for HistoryItemDto {
    fn from(info: UpdateInfo) -> Self {
        HistoryItemDto {
            version: info.clock as u64,
            updates: info.update.encode_v1(),
            timestamp: chrono::DateTime::from_timestamp(info.timestamp.unix_timestamp(), 0)
                .unwrap_or(Utc::now()),
        }
    }
}
