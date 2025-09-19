use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait DocumentStorageRepository: Send + Sync {
    /// Save document snapshot to storage
    async fn save_snapshot(&self, document_id: &str, data: &[u8]) -> Result<()>;

    /// Load document from storage
    async fn load_document(&self, document_id: &str) -> Result<Option<Vec<u8>>>;

    /// Flush updates to storage
    async fn flush_updates(&self, document_id: &str, updates: Vec<Vec<u8>>) -> Result<()>;
}
