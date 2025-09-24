use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct Document {
    pub id: String,
    pub updates: Vec<u8>,
    pub version: u64,
    pub timestamp: DateTime<Utc>,
}

impl Document {
    pub fn new(id: String, updates: Vec<u8>, version: u64, timestamp: DateTime<Utc>) -> Self {
        Self {
            id,
            updates,
            version,
            timestamp,
        }
    }
}
