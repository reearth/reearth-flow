use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct DocumentId(String);

impl DocumentId {
    pub fn new(id: String) -> Self {
        let normalized = id.strip_suffix(":main").unwrap_or(&id).to_string();
        DocumentId(normalized)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for DocumentId {
    fn from(id: String) -> Self {
        DocumentId::new(id)
    }
}

impl From<&str> for DocumentId {
    fn from(id: &str) -> Self {
        DocumentId::new(id.to_string())
    }
}

#[derive(Clone)]
pub struct Document {
    pub id: DocumentId,
    pub awareness: Arc<RwLock<yrs::sync::Awareness>>,
}

impl Document {
    pub fn new(id: DocumentId) -> Self {
        let doc = yrs::Doc::new();
        let awareness = yrs::sync::Awareness::new(doc);

        Self {
            id,
            awareness: Arc::new(RwLock::new(awareness)),
        }
    }
}
