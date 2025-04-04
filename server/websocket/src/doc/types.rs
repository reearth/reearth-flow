use chrono::Utc;
use serde::{Deserialize, Serialize};
use yrs::updates::encoder::Encode;
use crate::storage::gcs::UpdateInfo;

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

impl From<UpdateInfo> for HistoryItem {
    fn from(info: UpdateInfo) -> Self {
        HistoryItem {
            version: info.clock as u64,
            updates: info.update.encode_v1(),
            timestamp: chrono::DateTime::from_timestamp(
                info.timestamp.unix_timestamp(),
                0,
            )
            .unwrap_or(Utc::now()),
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
