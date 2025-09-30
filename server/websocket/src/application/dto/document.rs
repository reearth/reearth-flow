use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentDto {
    pub id: String,
    pub updates: Vec<u8>,
    pub version: u64,
    pub timestamp: String,
}
