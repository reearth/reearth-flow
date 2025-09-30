use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryItemDto {
    pub version: u64,
    pub timestamp: String,
    pub updates: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryMetadataDto {
    pub version: u64,
    pub timestamp: String,
}
