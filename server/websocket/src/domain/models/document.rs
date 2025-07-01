use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// 文档ID值对象
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct DocumentId(String);

impl DocumentId {
    pub fn new(id: String) -> Self {
        // 规范化文档ID，移除":main"后缀
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

/// 文档实体
#[derive(Clone)]
pub struct Document {
    pub id: DocumentId,
    pub awareness: Arc<RwLock<yrs::sync::Awareness>>,
    pub clock: u32,
}

impl Document {
    pub fn new(id: DocumentId) -> Self {
        let doc = yrs::Doc::new();
        let awareness = yrs::sync::Awareness::new(doc);

        Self {
            id,
            awareness: Arc::new(RwLock::new(awareness)),
            clock: 0,
        }
    }

    pub async fn update_clock(&mut self, new_clock: u32) -> Result<()> {
        self.clock = new_clock;
        Ok(())
    }
}
