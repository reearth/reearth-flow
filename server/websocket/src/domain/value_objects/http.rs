use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct HistoryItem {
    pub version: u64,
    pub updates: Vec<u8>,
    pub timestamp: DateTime<Utc>,
}

impl HistoryItem {
    pub fn new(version: u64, updates: Vec<u8>, timestamp: DateTime<Utc>) -> Self {
        Self {
            version,
            updates,
            timestamp,
        }
    }
}
