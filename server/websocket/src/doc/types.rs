#[derive(Debug, Clone)]
struct Document {
    pub id: String,
    pub updates: Vec<u8>,
    pub version: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
struct HistoryItem {
    pub version: u64,
    pub updates: Vec<u8>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

pub struct DocumentHandler {
    storage: Arc<GcsStore>,
    runtime: Handle,
}
